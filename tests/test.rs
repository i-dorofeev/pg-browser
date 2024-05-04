use std::fs;

use pg_browser::readers::database_file_layout::PGData;
use rstest::rstest;
use utils::fixture::*;

mod utils;

#[rstest]
fn my_test(pgdata: PGData) {
    let paths = fs::read_dir(pgdata.path()).unwrap();

    for path in paths {
        println!("Name: {}", path.unwrap().path().display())
    }
}