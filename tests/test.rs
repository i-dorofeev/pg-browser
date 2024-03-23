use std::{env, ffi::OsString, process::Command};
use colored::Colorize;
use merkle_hash::{MerkleTree, Encodable};


#[test]
fn build_docker_image() {
    let current_dir = env::current_dir().expect("current path");
    let docker_build_context = current_dir.join("postgres").into_os_string();
    // fs::create_dir_all("target/postgres").expect("dir created");

    run_command(Command::new("docker")
        .arg("build")
        .arg("-t")
        .arg("pg-browser/postgres")
        .arg(docker_build_context));
}

#[test]
fn hash_docker_build_context() {
    let tree = MerkleTree::builder(docker_build_context().into_string().unwrap()).build().unwrap();
    println!("{}", tree.root.item.hash.to_hex_string());
}

fn docker_build_context() -> OsString {
    let current_dir = env::current_dir().expect("current path");
    current_dir.join("postgres").into_os_string()
}

fn run_command(command: &mut Command) {
    println!("\n{:?}", &command);

    let output = command.output().expect("command run");

    println!("\n{}", String::from_utf8_lossy(&output.stdout));
    println!("{}", String::from_utf8_lossy(&output.stderr).yellow());

    if output.status.success() {
        println!("{}", "Success".green());
    } else if let Some(code) = output.status.code() {
        println!("{}", format!("Failed with {}", code).red());
    } else {
        println!("{}", "Failed".red());
    }
}
