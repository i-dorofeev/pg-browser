mod utils;

use pg_browser::readers::root_dir_readers::base_reader::{
    BaseDir, BaseDirItem, BaseDirReader, DatabaseDir,
};
use pg_browser::readers::{
    database_file_layout::PGData, root_dir_readers::base_reader::base_dir_reader,
};
use pretty_assertions::assert_eq;
use rstest::rstest;
use utils::fixture::*;

#[rstest]
fn reads_base_dir(pgdata: PGData) {
    // given
    let base = pgdata.base();
    let reader = base_dir_reader(base.path());

    // when
    let base = reader.read_base_dir().unwrap();

    // then
    assert_eq!(
        base,
        BaseDir(vec![
            BaseDirItem::DatabaseDir(DatabaseDir { name: "1".into() }),
            BaseDirItem::DatabaseDir(DatabaseDir { name: "4".into() }),
            BaseDirItem::DatabaseDir(DatabaseDir { name: "5".into() }),
        ])
    )
}
