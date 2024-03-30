use colored::Colorize;
use merkle_hash::{Encodable, MerkleTree};
use std::{
    env,
    ffi::OsString,
    fs,
    io::{BufRead, BufReader, Error, Read},
    process::{Child, Command, ExitStatus, Stdio},
    sync::{
        mpsc::{channel, Receiver, Sender, TryRecvError},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

#[test]
fn build_docker_image() {
    let current_dir = env::current_dir().expect("current path");
    let docker_build_context = current_dir.join("postgres").into_os_string();

    let mut command = Command::new("docker");
    command
        .arg("build")
        .arg("-t")
        .arg("pg-browser/postgres")
        .arg(docker_build_context);
    ChildProcess::start(command).listen(vec![&print]);
}

#[derive(PartialEq)]
enum ListenResult {
    Break,
    Continue,
}

fn print(output: &ChildProcessOutput) -> ListenResult {
    match output {
        ChildProcessOutput::None => {}

        ChildProcessOutput::StdErr(stderr) => {
            print!("{}", stderr.yellow());
        }

        ChildProcessOutput::StdOut(stdout) => {
            print!("{}", stdout);
        }

        ChildProcessOutput::Error(err) => {
            println!("{}", format!("{:?}", err).red());
        }

        ChildProcessOutput::Completed(exit_status) => {
            print_exit_status(exit_status);
        }
    }
    ListenResult::Continue
}

type Listener = dyn Fn(&ChildProcessOutput) -> ListenResult;

#[test]
fn hash_docker_build_context() {
    let tree = MerkleTree::builder(docker_build_context().into_string().unwrap())
        .build()
        .unwrap();
    println!("{}", tree.root.item.hash.to_hex_string());
}

#[test]
fn hash_pgdata() {
    let tree = MerkleTree::builder(pgdata_dir().into_string().unwrap())
        .build()
        .unwrap();
    println!("{}", tree.root.item.hash.to_hex_string());
}

// use merkle_hash::{MerkleItem, MerkleTree};
/*
docker run --rm --name pg-browser --user "$(id -u):$(id -g)" -e POSTGRES_PASSWORD=mysecretpassword -v ./target/pgdata:/var/lib/postgresql/data pg-browser/postgres
*/
#[test]
fn init_pgdata() {
    fs::create_dir_all(pgdata_dir()).expect("dir created");

    let uid = users::get_current_uid();
    let gid = users::get_current_gid();

    let mut command = Command::new("docker");
    command
        .arg("run")
        .arg("--rm")
        .arg("-it")
        .arg("--name")
        .arg("pg-browser")
        .arg("--user")
        .arg(format!("{uid}:{gid}"))
        .arg("-e")
        .arg("POSTGRES_PASSWORD=mysecretpassword")
        .arg("-v")
        .arg(format!(
            "{}:{}",
            pgdata_dir().to_str().unwrap(),
            "/var/lib/postgresql/data"
        ))
        .arg("pg-browser/postgres");

    ChildProcess::start(command).listen(vec![
        &print,
        // &until_stdout_logged("database system is ready to accept connections".to_string()),
    ]);
}

fn pgdata_dir() -> OsString {
    let current_dir = env::current_dir().expect("current path");
    current_dir.join("target/pgdata").into_os_string()
}

fn docker_build_context() -> OsString {
    let current_dir = env::current_dir().expect("current path");
    current_dir.join("postgres").into_os_string()
}

fn print_exit_status(status: &ExitStatus) {
    if status.success() {
        println!("{}", "Success".green());
    } else if let Some(code) = status.code() {
        println!("{}", format!("Failed with {}", code).red());
    } else {
        println!("{}", "Failed".red());
    }
}

struct ChildProcess {
    child: Child,
    output: Receiver<ChildProcessOutput>,
    stdout_thread: Option<JoinHandle<()>>,
    stderr_thread: Option<JoinHandle<()>>,
    exit_status: Arc<Mutex<Option<ExitStatus>>>,
}

impl ChildProcess {
    fn start(mut command: Command) -> Self {
        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let (tx, rx) = channel::<ChildProcessOutput>();
        let exit_status: Arc<Mutex<Option<ExitStatus>>> = Arc::new(Mutex::new(Option::None));

        let stdout = child.stdout.take().expect("stdout");
        let stdout_sender = tx.clone();
        let stdout_exit_status = Arc::clone(&exit_status);
        let stdout_thread = thread::spawn(move || {
            Self::read(stdout_exit_status, stdout, &stdout_sender, |result| {
                result.map_or_else(
                    |e| ChildProcessOutput::Error(ChildProcessError::StdOut(e)),
                    ChildProcessOutput::StdOut,
                )
            });
        });

        let stderr = child.stderr.take().expect("stderr");
        let stderr_sender = tx.clone();
        let stderr_exit_status = Arc::clone(&exit_status);
        let stderr_thread = thread::spawn(move || {
            Self::read(stderr_exit_status, stderr, &stderr_sender, |result| {
                result.map_or_else(
                    |e| ChildProcessOutput::Error(ChildProcessError::StdErr(e)),
                    ChildProcessOutput::StdErr,
                )
            });
        });

        ChildProcess {
            child,
            output: rx,
            stdout_thread: Some(stdout_thread),
            stderr_thread: Some(stderr_thread),
            exit_status,
        }
    }

    fn read<R, F>(
        exit_status: Arc<Mutex<Option<ExitStatus>>>,
        source: R,
        target: &Sender<ChildProcessOutput>,
        output: F,
    ) where
        R: Read,
        F: Fn(Result<String, Error>) -> ChildProcessOutput,
    {
        let mut reader = BufReader::new(source);
        let mut i = 2;
        while i > 0 {
            let mut buf = String::new();
            match reader.read_line(&mut buf) {
                Ok(_) => {
                    if !buf.is_empty() {
                        target.send(output(Ok(buf))).unwrap();
                    }
                }
                Err(err) => {
                    target.send(output(Err(err))).unwrap();
                }
            }

            let exit_status = exit_status.lock().unwrap();
            if exit_status.is_some() {
                i -= 1;
            }
        }
    }

    fn listen(&mut self, listeners: Vec<&Listener>) {
        loop {
            match self.receive() {
                completed @ ChildProcessOutput::Completed(_) => {
                    listeners.iter().for_each(|listener| {
                        listener(&completed);
                    });

                    break;
                }
                output => {
                    let r#break = listeners
                        .iter()
                        .map(|listener| listener(&output))
                        .any(|listen_result| listen_result == ListenResult::Break);
                    if r#break {
                        println!("Killing...");
                        self.child.kill().expect("child process failed to stop");
                        println!("Killed...");
                    }
                }
            }
        }
    }

    fn receive(&mut self) -> ChildProcessOutput {
        match self.output.try_recv() {
            Ok(output) => output,
            Err(TryRecvError::Empty) => self.check_terminated().map_or_else(
                |e| ChildProcessOutput::Error(ChildProcessError::Wait(e)),
                |_| ChildProcessOutput::None,
            ),
            Err(TryRecvError::Disconnected) => {
                self.join_threads();

                let exit_status = self.exit_status.lock().unwrap();
                ChildProcessOutput::Completed((*exit_status).unwrap())
            }
        }
    }

    fn check_terminated(&mut self) -> Result<(), Error> {
        let mut exit_status = self.exit_status.lock().unwrap();
        match *exit_status {
            Some(_) => Ok(()),
            None => match self.child.try_wait() {
                Ok(maybe_exit_status) => {
                    *exit_status = maybe_exit_status;
                    Ok(())
                }
                Err(e) => Err(e),
            },
        }
    }

    fn join_threads(&mut self) {
        if let Some(t) = self.stderr_thread.take() {
            t.join().unwrap();
        }
        if let Some(t) = self.stdout_thread.take() {
            t.join().unwrap();
        }
    }
}

enum ChildProcessOutput {
    None,
    StdErr(String),
    StdOut(String),
    Completed(ExitStatus),
    Error(ChildProcessError),
}

#[derive(Debug)]
enum ChildProcessError {
    Wait(Error),
    StdErr(Error),
    StdOut(Error),
}
