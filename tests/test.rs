use std::{
    env,
    fs::{create_dir_all, remove_dir_all},
    path::PathBuf,
    process::Command,
};
use utils::{child::start_child, fingerprint::Fingerprint};

mod utils;

const POSTGRES_IMAGE: &str = "pg-browser/postgres";

#[test]
fn ensure_pgdata() {
    let directory_hash = Fingerprint {
        paths: vec![docker_build_context_path(), pgdata_path()],
        hash_file: fingerprint_path(),
    };

    let hashes_match = directory_hash
        .load()
        .map(|stored_hash| directory_hash.compute() == stored_hash)
        .unwrap_or(false);

    if !hashes_match {
        println!("Rebuilding pgdata...");
        build_postgres_docker_image();
        init_pgdata();
        directory_hash.compute_and_store();
    }
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
    remove_dir_all(pgdata_path()).unwrap();
    create_dir_all(pgdata_path()).unwrap();

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
            pgdata_path().to_str().unwrap(),
            "/var/lib/postgresql/data"
        ))
        .arg(POSTGRES_IMAGE);

    start_child(command);
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
