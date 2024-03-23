use std::{env, process::Command};
use colored::Colorize;

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
