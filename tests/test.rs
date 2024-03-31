use std::{fs, path::PathBuf};

use rstest::{fixture, rstest};

mod utils;

#[fixture]
fn pgdata() -> PathBuf {
    utils::pgdata::ensure_pgdata()
}

#[rstest]
fn my_test(pgdata: PathBuf) {
    let paths = fs::read_dir(pgdata).unwrap();

    for path in paths {
        println!("Name: {}", path.unwrap().path().display())
    }
}
