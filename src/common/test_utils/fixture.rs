use std::path::PathBuf;

use rstest::fixture;

#[fixture]
pub fn pgdata() -> PathBuf {
    super::pgdata::ensure_pgdata()
}
