mod utils;

use pg_browser::readers::{
    database_file_layout::PGData,
    root_dir_reader::{
        root_dir_reader, PgDataItem, PgDataItemState, PgDataItemType, RootDirReader,
    },
};
use pretty_assertions::assert_eq;
use rstest::rstest;
use utils::fixture::*;

#[rstest]
fn reads_root_dir(pgdata: PGData) {
    // given
    let reader = root_dir_reader(pgdata.path());

    // when
    let items = reader.known_pgdata_items();

    // then
    assert_eq!(
        items,
        vec![
            file("PG_VERSION", "Major version number of PostgreSQL"),
            dir("base", "Per-database directories"),
            missing_file("current_logfiles", "File recording the log file(s) currently written to by the logging collector"),
            dir("global", "Cluster-wide tables, such as pg_database"),
            dir("pg_commit_ts", "Transaction commit timestamp data"),
            dir("pg_dynshmem", "Files used by the dynamic shared memory subsystem"),
            dir("pg_logical", "Status data for logical decoding"),
            dir("pg_multixact", "Multitransaction status data (used for shared row locks)"),
            dir("pg_notify", "LISTEN/NOTIFY status data"),
            dir("pg_replslot", "Replication slot data"),
            dir("pg_serial", "Information about committed serializable transactions"),
            dir("pg_snapshots", "Exported snapshots"),
            dir("pg_stat", "Permanent files for the statistics subsystem"),
            dir("pg_stat_tmp", "Temporary files for the statistics subsystem"),
            dir("pg_subtrans", "Subtransaction status data"),
            dir("pg_tblspc", "Symbolic links to tablespaces"),
            dir("pg_twophase", "State files for prepared transactions"),
            dir("pg_wal", "WAL (Write Ahead Log) files"),
            dir("pg_xact", "Transaction commit status data"),
            file("postgresql.auto.conf", "A file used for storing configuration parameters that are set by ALTER SYSTEM"),
            file("postmaster.opts", "A file recording the command-line options the server was last started with"),
            missing_file("postmaster.pid", "A lock file recording the current postmaster process ID (PID) and other running server data")
        ]
    );
}

fn dir(name: &'static str, description: &'static str) -> PgDataItem {
    PgDataItem {
        name,
        description,
        item_type: PgDataItemType::Dir,
        state: PgDataItemState::Present,
    }
}

fn missing_file(name: &'static str, description: &'static str) -> PgDataItem {
    PgDataItem {
        name,
        description,
        item_type: PgDataItemType::File,
        state: PgDataItemState::Missing,
    }
}

fn file(name: &'static str, description: &'static str) -> PgDataItem {
    PgDataItem {
        name,
        description,
        item_type: PgDataItemType::File,
        state: PgDataItemState::Present,
    }
}
