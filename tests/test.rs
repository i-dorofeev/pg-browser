use std::{fs, path::PathBuf};

use rstest::rstest;
use utils::fixture::*;

mod utils;

#[rstest]
fn my_test(pgdata: PathBuf) {
    let paths = fs::read_dir(pgdata).unwrap();

    for path in paths {
        println!("Name: {}", path.unwrap().path().display())
    }
}
