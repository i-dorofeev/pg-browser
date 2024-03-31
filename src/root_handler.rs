use std::{
    fs::{self, Metadata},
    io::ErrorKind::NotFound,
    path::PathBuf,
};

use colored::{Color, Colorize};

use crate::handler::{Handler, TermSize};

pub struct RootHandler {
    pub pgdata: PathBuf,
}

impl Handler for RootHandler {
    fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String> {
        match param {
            "a" => Ok(Box::new(AHandler {})),
            "b" => Ok(Box::new(BHandler {})),
            val => Ok(Box::new(ArbHandler {
                val: String::from(val),
            })),
        }
    }

    fn handle(&self, term_size: &TermSize) -> String {
        let pgdata_items = self.known_pgdata_items();
        let name_col_width = pgdata_items
            .iter()
            .map(|item| item.name.len())
            .max()
            .unwrap_or(0);

        let mut output = String::new();
        self.known_pgdata_items()
            .iter()
            .map(|item| Self::format_pgdata_item(item, name_col_width, term_size.cols))
            .for_each(|item_str| {
                output.push('\n');
                output.push_str(&item_str);
            });
        output
    }
}

impl RootHandler {
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
                    NotFound => PgDataItemState::Missing,
                    _ => PgDataItemState::Error(err),
                },
            },
        }
    }

    fn format_pgdata_item(
        item: &PgDataItem,
        name_col_width: usize,
        terminal_width: usize,
    ) -> String {
        let item_type = match &item.item_type {
            PgDataItemType::Dir => "D",
            PgDataItemType::File => "F",
        };
        let item_type_colored = match &item.state {
            PgDataItemState::Present => item_type.green(),
            PgDataItemState::Missing => item_type.yellow(),
            PgDataItemState::Error(_) => item_type.red(),
        };

        let padded_name = format!("{name: <width$}", name = item.name, width = name_col_width);
        let padded_name_colored = match item.state {
            PgDataItemState::Present => padded_name.blue(),
            PgDataItemState::Missing => padded_name.color(Color::TrueColor {
                r: 127,
                g: 127,
                b: 127,
            }),
            PgDataItemState::Error(_) => padded_name.red(),
        };

        let description_padding = item_type.len() + 1 + padded_name.len() + 1;
        let description_col_width = terminal_width - description_padding;

        let mut output = String::new();
        output.push_str(&format!(
            "{} {} {}",
            item_type_colored,
            padded_name_colored,
            Self::split(item.description, 0, description_col_width)
        ));

        (1..)
            .map(|n| Self::split(item.description, n, description_col_width))
            .take_while(|slice| !slice.is_empty())
            .for_each(|slice| {
                output.push_str(&format!(
                    "\n{padding: >padding_width$}{slice}",
                    padding = "",
                    padding_width = description_padding
                ));
            });

        output
    }

    fn split(str: &str, n: usize, size: usize) -> String {
        str.chars().skip(size * n).take(size).collect()
    }
}

enum PgDataItemState {
    Present,
    Missing,
    Error(std::io::Error),
}

enum PgDataItemType {
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

struct PgDataItem {
    name: &'static str,
    description: &'static str,
    item_type: PgDataItemType,
    state: PgDataItemState,
}

struct AHandler {}
impl Handler for AHandler {
    fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String> {
        Err(format!("AHandler: Unknown param {param}"))
    }

    fn handle(&self, _term_size: &TermSize) -> String {
        "Handled by AHandler".to_string()
    }
}

struct BHandler {}
impl Handler for BHandler {
    fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String> {
        Err(format!("BHandler: Unknown param {param}"))
    }

    fn handle(&self, _term_size: &TermSize) -> String {
        "Handled by BHandler".to_string()
    }
}

struct ArbHandler {
    val: String,
}
impl Handler for ArbHandler {
    fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String> {
        let this_val = &self.val;
        Ok(Box::from(ArbHandler {
            val: format!("{this_val}/{param}"),
        }))
    }

    fn handle(&self, _term_size: &TermSize) -> String {
        self.val.clone()
    }
}
