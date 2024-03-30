use child::ChildProcess;
use merkle_hash::{Encodable, MerkleTree};
use std::{env, ffi::OsString, fs, process::Command};

mod child;

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
    ChildProcess::start(command).listen(vec![&child::print]);
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

    ChildProcess::start(command).listen(vec![&child::print]);
}

fn pgdata_dir() -> OsString {
    let current_dir = env::current_dir().expect("current path");
    current_dir.join("target/pgdata").into_os_string()
}

fn docker_build_context() -> OsString {
    let current_dir = env::current_dir().expect("current path");
    current_dir.join("postgres").into_os_string()
}
