use std::{
    fs::{self, Metadata},
    path::Path,
};

#[derive(PartialEq, Debug)]
pub struct PgDataItem {
    pub name: &'static str,
    pub description: &'static str,
    pub item_type: PgDataItemType,
    pub state: PgDataItemState,
}

#[derive(Debug)]
pub enum PgDataItemState {
    Present,
    Missing,
    Error(std::io::Error),
}

impl PartialEq for PgDataItemState {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

#[derive(PartialEq, Debug)]
pub enum PgDataItemType {
    File,
    Dir,
}

impl PgDataItemType {
    fn matches(&self, metadata: &Metadata) -> bool {
        match self {
            PgDataItemType::File => metadata.is_file(),
            PgDataItemType::Dir => metadata.is_dir(),
        }
    }
}

pub trait RootDirReader {
    fn known_pgdata_items(&self) -> Vec<PgDataItem>;
}

pub fn root_dir_reader(pgdata: &Path) -> impl RootDirReader + '_ {
    DefaultRootDirReader { pgdata }
}

struct DefaultRootDirReader<'a> {
    pub pgdata: &'a Path,
}

impl<'a> RootDirReader for DefaultRootDirReader<'a> {
    fn known_pgdata_items(&self) -> Vec<PgDataItem> {
        vec![
            self.file("PG_VERSION", "Major version number of PostgreSQL"),
            self.dir("base", "Per-database directories"),
            self.file("current_logfiles", "File recording the log file(s) currently written to by the logging collector"),
            self.dir("global", "Cluster-wide tables, such as pg_database"),
            self.dir("pg_commit_ts", "Transaction commit timestamp data"),
            self.dir("pg_dynshmem", "Files used by the dynamic shared memory subsystem"),
            self.dir("pg_logical", "Status data for logical decoding"),
            self.dir("pg_multixact", "Multitransaction status data (used for shared row locks)"),
            self.dir("pg_notify", "LISTEN/NOTIFY status data"),
            self.dir("pg_replslot", "Replication slot data"),
            self.dir("pg_serial", "Information about committed serializable transactions"),
            self.dir("pg_snapshots", "Exported snapshots"),
            self.dir("pg_stat", "Permanent files for the statistics subsystem"),
            self.dir("pg_stat_tmp", "Temporary files for the statistics subsystem"),
            self.dir("pg_subtrans", "Subtransaction status data"),
            self.dir("pg_tblspc", "Symbolic links to tablespaces"),
            self.dir("pg_twophase", "State files for prepared transactions"),
            self.dir("pg_wal", "WAL (Write Ahead Log) files"),
            self.dir("pg_xact", "Transaction commit status data"),
            self.file("postgresql.auto.conf", "A file used for storing configuration parameters that are set by ALTER SYSTEM"),
            self.file("postmaster.opts", "A file recording the command-line options the server was last started with"),
            self.file("postmaster.pid", "A lock file recording the current postmaster process ID (PID) and other running server data")
        ]
    }
}

impl<'a> DefaultRootDirReader<'a> {
    fn file(&self, name: &'static str, description: &'static str) -> PgDataItem {
        self.pgdata_item(PgDataItemType::File, name, description)
    }

    fn dir(&self, name: &'static str, description: &'static str) -> PgDataItem {
        self.pgdata_item(PgDataItemType::Dir, name, description)
    }

    fn pgdata_item(
        &self,
        item_type: PgDataItemType,
        name: &'static str,
        description: &'static str,
    ) -> PgDataItem {
        let item_path = self.pgdata.join(name);
        match fs::metadata(item_path) {
            Ok(metadata) => {
                let state = match &item_type.matches(&metadata) {
                    true => PgDataItemState::Present,
                    false => PgDataItemState::Missing,
                };
                PgDataItem {
                    name,
                    description,
                    item_type,
                    state,
                }
            }
            Err(err) => PgDataItem {
                name,
                description,
                item_type,
                state: match err.kind() {
                    std::io::ErrorKind::NotFound => PgDataItemState::Missing,
                    _ => PgDataItemState::Error(err),
                },
            },
        }
    }
}
