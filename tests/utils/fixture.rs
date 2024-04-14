use pg_browser::readers::database_file_layout::PGData;
use rstest::fixture;

#[fixture]
pub fn pgdata() -> PGData {
    let path = super::pgdata::ensure_pgdata();
    PGData(path)
}
