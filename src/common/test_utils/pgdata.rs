use std::{
    env,
    fs::{create_dir_all, remove_dir_all},
    io::{
        Error,
        ErrorKind::{self, NotFound},
    },
    path::PathBuf,
    process::Command,
};

use anyhow::anyhow;

use super::{child::start_child, fingerprint::Fingerprint};

const POSTGRES_IMAGE: &str = "pg-browser/postgres";

pub fn ensure_pgdata() -> PathBuf {
    let directory_hash = Fingerprint {
        paths: vec![docker_build_context_path(), pgdata_path()],
        hash_file: fingerprint_path(),
    };

    if is_pgdata_empty() || !directory_hash.stored_matches_actual() {
        println!("Rebuilding pgdata...");
        build_postgres_docker_image();
        init_pgdata();
        directory_hash.compute_and_store();
    }

    pgdata_path()
}

fn is_pgdata_empty() -> bool {
    let maybe_pgdata = std::fs::read_dir(pgdata_path())
        .map_or_else(expect(NotFound), ok_some())
        .expect("PGData dir read");

    maybe_pgdata.map_or(true, |mut pgdata| pgdata.next().is_none())
}

fn build_postgres_docker_image() {
    let mut command = Command::new("docker");
    command
        .arg("build")
        .arg("-t")
        .arg(POSTGRES_IMAGE)
        .arg(docker_build_context_path());
    start_child(command);
}

fn init_pgdata() {
    remove_dir_all(pgdata_path())
        .map_or_else(expect(NotFound), ok_some())
        .expect("PGData dir removed");
    create_dir_all(pgdata_path()).expect("PGData dir created");

    let uid = users::get_current_uid();
    let gid = users::get_current_gid();

    let mut command = Command::new("docker");
    command
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
            pgdata_path().to_str().unwrap(),
            "/var/lib/postgresql/data"
        ))
        .arg(POSTGRES_IMAGE);

    start_child(command);

    let mut pgdata = std::fs::read_dir(pgdata_path()).unwrap();
    let _ = pgdata
        .next()
        .ok_or(anyhow!("PGDATA hasn't been initialised"))
        .unwrap();
}

fn pgdata_path() -> PathBuf {
    let current_dir = env::current_dir().unwrap();
    current_dir.join("target/pgdata")
}

fn docker_build_context_path() -> PathBuf {
    let current_dir = env::current_dir().unwrap();
    current_dir.join("postgres")
}

fn fingerprint_path() -> PathBuf {
    let current_dir = env::current_dir().unwrap();
    current_dir.join("target/fingerprint")
}

fn expect<T>(error_kind: ErrorKind) -> impl FnOnce(Error) -> Result<Option<T>, Error> {
    return move |err| match err.kind() {
        kind if kind == error_kind => Ok(None),
        _ => Err(err),
    };
}

fn ok_some<T>() -> impl FnOnce(T) -> Result<Option<T>, Error> {
    return |v| Ok(Some(v));
}
