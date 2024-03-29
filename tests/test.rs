use colored::Colorize;
use merkle_hash::{Encodable, MerkleTree};
use std::{
    env,
    ffi::OsString,
    fs,
    io::{BufReader, Error, Read},
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
    let mut child = ChildProcess::start(command);
    loop {
        match child.receive() {
            ChildProcessOutput::StdErr(stderr) => {
                println!("{}", stderr.yellow());
            }
            ChildProcessOutput::StdOut(stdout) => {
                println!("{}", stdout);
            }
            ChildProcessOutput::Completed(exit_status) => {
                print_exit_status(&exit_status);
                break;
            }
            ChildProcessOutput::None => {}

            ChildProcessOutput::Error(err) => {
                println!("{}", format!("{:?}", err).red());
            }
        }
    }
}

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
// `docker run --rm --name pg-browser --user "$(id -u):$(id -g)" -e POSTGRES_PASSWORD=mysecretpassword
// -v ./target/pgdata:/var/lib/postgresql/data pg-browser/postgres`
#[test]
fn init_pgdata() {
    fs::create_dir_all(pgdata_dir()).expect("dir created");

    let uid = users::get_current_uid();
    let gid = users::get_current_gid();
    run_command(
        Command::new("docker")
            .arg("run")
            .arg("--rm")
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
            .arg("pg-browser/postgres"),
    );
}

fn pgdata_dir() -> OsString {
    let current_dir = env::current_dir().expect("current path");
    current_dir.join("target/pgdata").into_os_string()
}

fn docker_build_context() -> OsString {
    let current_dir = env::current_dir().expect("current path");
    current_dir.join("postgres").into_os_string()
}

fn run_command(command: &mut Command) {
    // println!("\n{:?}", &command);

    // let mut child = command
    //     .stdout(io::stdout())
    //     .stderr(io::stderr())
    //     .spawn()
    //     .unwrap();

    // {
    //     // let mut stdout = StdReader::stdout_reader(&child);
    //     // let mut stderr = StdReader::stderr_reader(&child);

    //     loop {
    //         // let stdout_complete = stdout.read_next_line(|line| {
    //         //     // listeners
    //         //     //     .iter()
    //         //     //     .for_each(|l| l.on_stdout_line(&mut child, &line));
    //         // });

    //         let stderr_complete = stderr.read_next_line(|line| {
    //             // listeners
    //             //     .iter()
    //             //     .for_each(|l| l.on_stderr_line(&mut child, &line));
    //         });

    //         if stdout_complete && stderr_complete {
    //             break;
    //         }
    //     }
    // }

    // match child.wait() {
    //     Ok(status) => print_exit_status(&status),
    //     Err(_) => println!("Error on process exit"),
    // }
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
    command: Command,
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
            command,
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
            match reader.read_to_string(&mut buf) {
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

    fn receive(&mut self) -> ChildProcessOutput {
        match self.output.try_recv() {
            Ok(output) => output,
            Err(TryRecvError::Empty) => {
                let output = {
                    let mut exit_status = self.exit_status.lock().unwrap();
                    match *exit_status {
                        Some(_) => ChildProcessOutput::None,
                        None => match self.child.try_wait() {
                            Ok(maybe_exit_status) => {
                                *exit_status = maybe_exit_status;
                                ChildProcessOutput::None
                            }
                            Err(e) => ChildProcessOutput::Error(ChildProcessError::Wait(e)),
                        },
                    }
                };

                output
            }
            Err(TryRecvError::Disconnected) => {
                if let Some(t) = self.stderr_thread.take() {
                    t.join().unwrap()
                }
                if let Some(t) = self.stdout_thread.take() {
                    t.join().unwrap()
                }

                let exit_status = self.exit_status.lock().unwrap();
                ChildProcessOutput::Completed((*exit_status).unwrap())
            }
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
