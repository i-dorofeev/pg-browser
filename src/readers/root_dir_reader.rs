use std::{
    fs::{self, Metadata},
    path::PathBuf,
};

pub struct PgDataItem {
    pub name: &'static str,
    pub description: &'static str,
    pub item_type: PgDataItemType,
    pub state: PgDataItemState,
}

pub enum PgDataItemState {
    Present,
    Missing,
    Error(std::io::Error),
}

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

pub trait RootDirReader<'a> {
    fn known_pgdata_items(&self) -> Vec<PgDataItem>;
}

struct DefaultRootDirReader<'a> {
    pub pgdata: &'a PathBuf,
}

pub fn default_root_dir_reader<'a>(pgdata: &'a PathBuf) -> impl RootDirReader<'a> {
    DefaultRootDirReader { pgdata }
}

impl<'a> RootDirReader<'a> for DefaultRootDirReader<'a> {
    fn known_pgdata_items(&self) -> Vec<PgDataItem> {
        vec![
            self.pgdata_file("PG_VERSION", "Major version number of PostgreSQL"),
            self.pgdata_dir("base", "Per-database directories"),
            self.pgdata_file(
                "current_logfiles",
                "File recording the log file(s) currently written to by the logging collector",
            ),
            self.pgdata_dir("global", "Cluster-wide tables, such as pg_database"),
            self.pgdata_dir("pg_commit_ts", "Transaction commit timestamp data"),
            self.pgdata_dir(
                "pg_dynshmem",
                "Files used by the dynamic shared memory subsystem",
            ),
            self.pgdata_dir("pg_logical", "Status data for logical decoding"),
            self.pgdata_dir(
                "pg_multixact",
                "Multitransaction status data (used for shared row locks)",
            ),
            self.pgdata_dir("pg_notify", "LISTEN/NOTIFY status data"),
            self.pgdata_dir("pg_replslot", "Replication slot data"),
            self.pgdata_dir(
                "pg_serial",
                "Information about committed serializable transactions",
            ),
            self.pgdata_dir("pg_snapshots", "Exported snapshots"),
            self.pgdata_dir("pg_stat", "Permanent files for the statistics subsystem"),
            self.pgdata_dir(
                "pg_stat_tmp",
                "Temporary files for the statistics subsystem",
            ),
            self.pgdata_dir("pg_subtrans", "Subtransaction status data"),
            self.pgdata_dir("pg_tblspc", "Symbolic links to tablespaces"),
            self.pgdata_dir("pg_twophase", "State files for prepared transactions"),
            self.pgdata_dir("pg_wal", "WAL (Write Ahead Log) files"),
            self.pgdata_dir("pg_xact", "Transaction commit status data"),
            self.pgdata_file("postgresql.auto.conf", "A file used for storing configuration parameters that are set by ALTER SYSTEM"),
            self.pgdata_file("postmaster.opts", "A file recording the command-line options the server was last started with"),
            self.pgdata_file("postmaster.pid", "A lock file recording the current postmaster process ID (PID) and other running server data")
        ]
    }
}

impl<'a> DefaultRootDirReader<'a> {
    fn pgdata_file(&self, name: &'static str, description: &'static str) -> PgDataItem {
        self.pgdata_item(PgDataItemType::File, name, description)
    }

    fn pgdata_dir(&self, name: &'static str, description: &'static str) -> PgDataItem {
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
