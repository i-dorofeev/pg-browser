use rstest::fixture;

use std::path::PathBuf;

#[fixture]
pub fn pgdata() -> PathBuf {
    super::pgdata::ensure_pgdata()
}
