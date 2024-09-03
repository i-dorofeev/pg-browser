use std::{borrow::Cow, ffi::OsStr};

use anyhow::anyhow;
use colored::Colorize;

use crate::{
    common::fs::{DirEntry, FileType},
    pgdata::{KnownPGDataItem, PGData, PGDataItem, PGDataItemState, PGDataItems},
    GRAY,
};

use self::base::BaseHandler;

use super::{Viewer, TermSize};

mod base;

pub struct RootViewer<T: PGData> {
    // TODO: create factory and make private
    pub pgdata: T,
}

impl<T: PGData> Viewer for RootViewer<T> {
    fn get_next(self: Box<Self>, param: &str) -> anyhow::Result<Box<dyn Viewer>> {
        match param {
            "base" => Ok(Box::new(BaseHandler {
                base: self.pgdata.items().base(),
            })),
            "a" => Ok(Box::new(AHandler {})),
            "b" => Ok(Box::new(BHandler {})),
            val => Ok(Box::new(ArbHandler {
                val: String::from(val),
            })),
        }
    }

    fn handle<'a>(
        &self,
        term_size: &'a TermSize,
        mut write: Box<&mut dyn std::io::Write>,
    ) -> anyhow::Result<()> {
        let pgdata_item_intoiter = self.pgdata.list_items()?;

        let pgdata_item_iter = pgdata_item_intoiter.into_iter();

        let pgdata_item_views = pgdata_item_iter
            .map(|pgdata_item| PgDataItemView::from(pgdata_item))
            .collect::<Vec<_>>();

        let name_col_width = pgdata_item_views
            .iter()
            .map(|item| item.name.len())
            .max()
            .unwrap_or(0);

        pgdata_item_views
            .into_iter()
            .enumerate()
            .map(|(i, item)| {
                if i > 0 {
                    write.write("\n".as_bytes())?;
                }
                item.write(name_col_width, term_size.cols, &mut write)
            })
            .collect()
    }
}

#[derive(PartialEq, Debug)]
struct PgDataItemView<'a> {
    name: Cow<'a, OsStr>,
    description: Cow<'a, str>,
    item_type: FileType,
    state: PGDataItemState,
}

impl PgDataItemView<'_> {
    fn from<'a>(pgdata_item: PGDataItem<'a>) -> PgDataItemView<'a> {
        match pgdata_item {
            PGDataItem::Known(dir_entry, known_pgdata_item, state) => {
                PgDataItemView::known(dir_entry, known_pgdata_item, state)
            }
            PGDataItem::Unknown(dir_entry) => PgDataItemView::unknown(dir_entry),
        }
    }

    fn unknown<'a>(dir_entry: DirEntry<'a>) -> PgDataItemView<'a> {
        PgDataItemView {
            name: dir_entry.name,
            description: "".into(),
            item_type: dir_entry.entry_type,
            state: PGDataItemState::Present,
        }
    }

    fn known<'a>(
        dir_entry: DirEntry<'a>,
        known_pgdata_item: KnownPGDataItem,
        state: PGDataItemState,
    ) -> PgDataItemView<'a> {
        let description = match known_pgdata_item {
            KnownPGDataItem::PGVersion => "Major version number of PostgreSQL",
            KnownPGDataItem::Base =>  "Per-database directories",
            KnownPGDataItem::CurrentLogFiles => "File recording the log file(s) currently written to by the logging collector",
            KnownPGDataItem::Global =>  "Subdirectory containing cluster-wide tables, such as pg_database",
            KnownPGDataItem::PGCommitTS => "Transaction commit timestamp data",
            KnownPGDataItem::PGDynshmem => "Files used by the dynamic shared memory subsystem",
            KnownPGDataItem::PGLogical => "Status data for logical decoding",
            KnownPGDataItem::PGMultixact => "Multitransaction status data (used for shared row locks)",
            KnownPGDataItem::PGNotify => "LISTEN/NOTIFY status data",
            KnownPGDataItem::PGReplslot => "Replication slot data",
            KnownPGDataItem::PGSerial => "Information about committed serializable transactions",
            KnownPGDataItem::PGSnapshots => "Exported snapshots",
            KnownPGDataItem::PGStat => "Permanent files for the statistics subsystem",
            KnownPGDataItem::PGStatTmp => "Temporary files for the statistics subsystem",
            KnownPGDataItem::PGSubtrans => "Subtransaction status data",
            KnownPGDataItem::PGTblspc => "Symbolic links to tablespaces",
            KnownPGDataItem::PGTwophase => "State files for prepared transactions",
            KnownPGDataItem::PGWal => "WAL (Write Ahead Log) files",
            KnownPGDataItem::PGXact => "Transaction commit status data",
            KnownPGDataItem::PostgresqlConf => "A file used for storing configuration parameters that are set manually by system administrator",
            KnownPGDataItem::PostgresqlAutoConf => "A file used for storing configuration parameters that are set by ALTER SYSTEM",
            KnownPGDataItem::PostmasterOpts => "A file recording the command-line options the server was last started with",
            KnownPGDataItem::PostmasterPid => "A lock file recording the current postmaster process ID (PID) and other running server data",
            KnownPGDataItem::PGHbaConf => "Client authentication configuration file",
            KnownPGDataItem::PGIdentConf => "User name mappings for external authentication system",
        };

        PgDataItemView {
            name: dir_entry.name,
            description: description.into(),
            item_type: dir_entry.entry_type,
            state,
        }
    }

    fn write(
        &self,
        name_col_width: usize,
        terminal_width: usize,
        target: &mut Box<&mut dyn std::io::Write>,
    ) -> anyhow::Result<()> {
        let item_type = match self.item_type {
            FileType::Dir => "D",
            FileType::File => "F",
        };
        let item_type_colored = match self.state {
            PGDataItemState::Present => item_type.green(),
            PGDataItemState::Missing => item_type.yellow(),
        };

        let padded_name = format!(
            "{name: <width$}",
            name = self.name.to_string_lossy(),
            width = name_col_width
        );
        let padded_name_colored = match self.state {
            PGDataItemState::Present => padded_name.blue(),
            PGDataItemState::Missing => padded_name.color(GRAY),
        };

        let description_padding = item_type.len() + 1 + padded_name.len() + 1;
        let description_col_width = terminal_width - description_padding;

        write!(target, "{} {} ", item_type_colored, padded_name_colored)?;

        let description_chunks = Self::chunks(&self.description, description_col_width);
        description_chunks
            .enumerate()
            .map(|(i, chunk)| {
                if i != 0 {
                    write!(target, "\n{0: >1$}", " ", description_padding)?;
                }
                write!(target, "{}", chunk)?;
                Ok(())
            })
            .collect()
    }

    fn chunks(str: &str, size: usize) -> StrChunks {
        StrChunks {
            chunk_size: size,
            str: str,
        }
    }
}

struct StrChunks<'a> {
    chunk_size: usize,
    str: &'a str,
}

impl<'a> Iterator for StrChunks<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.str.is_empty() {
            return None;
        }

        self.str
            .char_indices()
            .map(|(i, _ch)| i)
            .take(self.chunk_size)
            .last()
            .map(|i| self.str.split_at(i + 1))
            .filter(|(left, _right)| !left.is_empty())
            .inspect(|(_left, right)| self.str = right.trim())
            .map(|(left, _right)| left)
    }
}

struct AHandler {}
impl Viewer for AHandler {
    fn get_next(self: Box<Self>, param: &str) -> anyhow::Result<Box<dyn Viewer>> {
        Err(anyhow!("AHandler: Unknown param {param}"))
    }

    fn handle<'a>(
        &self,
        _term_size: &'a TermSize,
        write: Box<&mut dyn std::io::Write>,
    ) -> anyhow::Result<()> {
        writeln!(write, "Handled by AHandler").map_err(|err| anyhow!(err))
    }
}

struct BHandler {}
impl Viewer for BHandler {
    fn get_next(self: Box<Self>, param: &str) -> anyhow::Result<Box<dyn Viewer>> {
        Err(anyhow!("BHandler: Unknown param {param}"))
    }

    fn handle<'a>(
        &self,
        _term_size: &'a TermSize,
        write: Box<&mut dyn std::io::prelude::Write>,
    ) -> anyhow::Result<()> {
        writeln!(write, "Handled by BHandler").map_err(|err| anyhow!(err))
    }
}

struct ArbHandler {
    val: String,
}
impl Viewer for ArbHandler {
    fn get_next(self: Box<Self>, param: &str) -> anyhow::Result<Box<dyn Viewer>> {
        let this_val = &self.val;
        Ok(Box::from(ArbHandler {
            val: format!("{this_val}/{param}"),
        }))
    }

    fn handle<'a>(
        &self,
        _term_size: &'a TermSize,
        write: Box<&mut dyn std::io::Write>,
    ) -> anyhow::Result<()> {
        writeln!(write, "{}", self.val.clone()).map_err(|err| anyhow!(err))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use pretty_assertions::assert_eq;
    use strum::IntoEnumIterator;

    use crate::{
        pgdata::{test_stubs::StubPGDataItems, KnownPGDataItem, PGData, PGDataItem, PGDataItems},
        test_utils::{
            colors::{BLUE, GREEN, NONE},
            line,
        }, viewers::{Viewer, TermSize},
    };

    use super::RootViewer;

    struct StubPGData<F>
    where
        F: Fn() -> anyhow::Result<Vec<PGDataItem<'static>>>,
    {
        path: &'static Path,
        items: F,
    }

    impl<F> PGData for StubPGData<F>
    where
        F: Fn() -> anyhow::Result<Vec<PGDataItem<'static>>>,
    {
        fn path(&self) -> &Path {
            &self.path
        }

        fn list_items(&self) -> anyhow::Result<Vec<PGDataItem>> {
            (self.items)()
        }

        fn items(&self) -> impl PGDataItems {
            StubPGDataItems {}
        }
    }

    #[test]
    fn root_hander_renders_root_dir_contents() {
        // given
        let root_handler = RootViewer {
            pgdata: StubPGData {
                path: Path::new("/pgdata"),
                items: || root_dir_items(),
            },
        };

        let term_size = TermSize {
            rows: 100,
            cols: 80,
        };

        let mut buf = Vec::new();

        // when
        root_handler.handle(&term_size, Box::new(&mut buf)).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // then
        #[rustfmt::skip]
        assert_eq!(
            output,
            [
                line("F| |pgversion           | |Major version number of PostgreSQL",                        &[ GREEN, NONE, BLUE, NONE, NONE]),
                line("D| |base                | |Per-database directories",                                  &[ GREEN, NONE, BLUE, NONE, NONE]),
                line("D| |current_log_files   | |File recording the log file(s) currently written to by th", &[ GREEN, NONE, BLUE, NONE, NONE]),
                line(" | |                    | |e logging collector",                                       &[  NONE, NONE, NONE, NONE, NONE]),
                line("D| |global              | |Subdirectory containing cluster-wide tables, such as pg_d", &[ GREEN, NONE, BLUE, NONE, NONE]),
                line(" | |                    | |atabase",                                                   &[  NONE, NONE, NONE, NONE, NONE]),
                line("D| |pg_commit_ts        | |Transaction commit timestamp data",                         &[ GREEN, NONE, BLUE, NONE, NONE]),
                line("D| |pg_dynshmem         | |Files used by the dynamic shared memory subsystem",         &[ GREEN, NONE, BLUE, NONE, NONE]),
                line("D| |pg_logical          | |Status data for logical decoding",                          &[ GREEN, NONE, BLUE, NONE, NONE]),
                line("D| |pg_multixact        | |Multitransaction status data (used for shared row locks)",  &[ GREEN, NONE, BLUE, NONE, NONE]),
                line("D| |pg_notify           | |LISTEN/NOTIFY status data",                                 &[ GREEN, NONE, BLUE, NONE, NONE]),
                line("D| |pg_replslot         | |Replication slot data",                                     &[ GREEN, NONE, BLUE, NONE, NONE]),
                line("D| |pg_serial           | |Information about committed serializable transactions",     &[ GREEN, NONE, BLUE, NONE, NONE]),
                line("D| |pg_snapshots        | |Exported snapshots",                                        &[ GREEN, NONE, BLUE, NONE, NONE]),
                line("D| |pg_stat             | |Permanent files for the statistics subsystem",              &[ GREEN, NONE, BLUE, NONE, NONE]),
                line("D| |pg_stat_tmp         | |Temporary files for the statistics subsystem",              &[ GREEN, NONE, BLUE, NONE, NONE]),
                line("D| |pg_subtrans         | |Subtransaction status data",                                &[ GREEN, NONE, BLUE, NONE, NONE]),
                line("D| |pg_tblspc           | |Symbolic links to tablespaces",                             &[ GREEN, NONE, BLUE, NONE, NONE]),
                line("D| |pg_twophase         | |State files for prepared transactions",                     &[ GREEN, NONE, BLUE, NONE, NONE]),
                line("D| |pg_wal              | |WAL (Write Ahead Log) files",                               &[ GREEN, NONE, BLUE, NONE, NONE]),
                line("D| |pg_xact             | |Transaction commit status data",                            &[ GREEN, NONE, BLUE, NONE, NONE]),
                line("F| |postgresql.conf     | |A file used for storing configuration parameters that are", &[ GREEN, NONE, BLUE, NONE, NONE]),
                line(" | |                    | |set manually by system administrator",                      &[  NONE, NONE, NONE, NONE, NONE]),
                line("F| |postgresql.auto.conf| |A file used for storing configuration parameters that are", &[ GREEN, NONE, BLUE, NONE, NONE]),
                line(" | |                    | |set by ALTER SYSTEM",                                       &[  NONE, NONE, NONE, NONE, NONE]),
                line("F| |postmaster.opts     | |A file recording the command-line options the server was ", &[ GREEN, NONE, BLUE, NONE, NONE]),
                line(" | |                    | |last started with",                                         &[  NONE, NONE, NONE, NONE, NONE]),
                line("F| |postmaster.pid      | |A lock file recording the current postmaster process ID (", &[ GREEN, NONE, BLUE, NONE, NONE]),
                line(" | |                    | |PID) and other running server data",                        &[  NONE, NONE, NONE, NONE, NONE]),
                line("F| |pg_hba.conf         | |Client authentication configuration file",                  &[ GREEN, NONE, BLUE, NONE, NONE]),
                line("F| |pg_ident.conf       | |User name mappings for external authentication system",     &[ GREEN, NONE, BLUE, NONE, NONE]),
            ]
            .join("\n")
        );
    }

    #[rustfmt::skip]
    fn root_dir_items() -> anyhow::Result<Vec<PGDataItem<'static>>> {
        let items = KnownPGDataItem::iter()
            .map(|known_pgdata_item| match known_pgdata_item {
                KnownPGDataItem::PGVersion => {
                    PGDataItem::known_present_file("pgversion", known_pgdata_item)
                }
                KnownPGDataItem::Base => {
                    PGDataItem::known_present_dir("base", known_pgdata_item)
                }
                KnownPGDataItem::CurrentLogFiles => {
                    PGDataItem::known_present_dir("current_log_files", known_pgdata_item)
                }
                KnownPGDataItem::Global => {
                    PGDataItem::known_present_dir("global", known_pgdata_item)
                }
                KnownPGDataItem::PGCommitTS => {
                    PGDataItem::known_present_dir("pg_commit_ts", known_pgdata_item)
                }
                KnownPGDataItem::PGDynshmem => {
                    PGDataItem::known_present_dir("pg_dynshmem", known_pgdata_item)
                }
                KnownPGDataItem::PGLogical => {
                    PGDataItem::known_present_dir("pg_logical", known_pgdata_item)
                }
                KnownPGDataItem::PGMultixact => {
                    PGDataItem::known_present_dir("pg_multixact", known_pgdata_item)
                }
                KnownPGDataItem::PGNotify => {
                    PGDataItem::known_present_dir("pg_notify", known_pgdata_item)
                }
                KnownPGDataItem::PGReplslot => {
                    PGDataItem::known_present_dir("pg_replslot", known_pgdata_item)
                }
                KnownPGDataItem::PGSerial => {
                    PGDataItem::known_present_dir("pg_serial", known_pgdata_item)
                }
                KnownPGDataItem::PGSnapshots => {
                    PGDataItem::known_present_dir("pg_snapshots", known_pgdata_item)
                }
                KnownPGDataItem::PGStat => {
                    PGDataItem::known_present_dir("pg_stat", known_pgdata_item)
                }
                KnownPGDataItem::PGStatTmp => {
                    PGDataItem::known_present_dir("pg_stat_tmp", known_pgdata_item)
                }
                KnownPGDataItem::PGSubtrans => {
                    PGDataItem::known_present_dir("pg_subtrans", known_pgdata_item)
                }
                KnownPGDataItem::PGTblspc => {
                    PGDataItem::known_present_dir("pg_tblspc", known_pgdata_item)
                }
                KnownPGDataItem::PGTwophase => {
                    PGDataItem::known_present_dir("pg_twophase", known_pgdata_item)
                }
                KnownPGDataItem::PGWal => {
                    PGDataItem::known_present_dir("pg_wal", known_pgdata_item)
                }
                KnownPGDataItem::PGXact => {
                    PGDataItem::known_present_dir("pg_xact", known_pgdata_item)
                }
                KnownPGDataItem::PostgresqlConf => {
                    PGDataItem::known_present_file("postgresql.conf", known_pgdata_item)
                }
                KnownPGDataItem::PostgresqlAutoConf => {
                    PGDataItem::known_present_file("postgresql.auto.conf", known_pgdata_item)
                }
                KnownPGDataItem::PostmasterOpts => {
                    PGDataItem::known_present_file("postmaster.opts", known_pgdata_item)
                }
                KnownPGDataItem::PostmasterPid => {
                    PGDataItem::known_present_file("postmaster.pid", known_pgdata_item)
                }
                KnownPGDataItem::PGHbaConf => {
                    PGDataItem::known_present_file("pg_hba.conf", known_pgdata_item)
                }
                KnownPGDataItem::PGIdentConf => {
                    PGDataItem::known_present_file("pg_ident.conf", known_pgdata_item)
                }
            })
            .collect::<Vec<_>>();
        Ok(items)
    }
}
