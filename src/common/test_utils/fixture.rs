use std::path::PathBuf;

use fslock::LockFile;
use rstest::fixture;

#[fixture]
pub fn pgdata() -> PathBuf {
    let mut lock_file = LockFile::open("target/pgdata_init.lock").unwrap();
    lock_file.lock().unwrap();

    super::pgdata::ensure_pgdata()
}
