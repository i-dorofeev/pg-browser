mod base;
mod current_logfiles;
mod global;
mod pg_commit_ts;
mod pg_dynshmem;
mod pg_hba_conf;
mod pg_ident_conf;
mod pg_logical;
mod pg_multiexact;
mod pg_notify;
mod pg_replslot;
mod pg_serial;
mod pg_snapshots;
mod pg_stat;
mod pg_stat_tmp;
mod pg_subtrans;
mod pg_tblspc;
mod pg_twophase;
mod pg_version;
mod pg_wal;
mod pg_xact;
mod postgresql_auto_conf;
mod postgresql_conf;
mod postmaster_opts;
mod postmaster_pid;

use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use crate::common::fs::DirEntry;

use self::base::Base;

/// Represents the PG_DATA directory
pub trait PGData {
    /// Returns the actual path of the directory represented by this trait
    fn path(&self) -> &Path;

    /// Lists all expected and unexpected items of PG_DATA
    fn items(&self) -> anyhow::Result<Box<dyn Iterator<Item = PGDataItem>>>;

    /// Represents PG_DATA/base directory
    fn base(&self) -> Box<dyn Base>;
}

/// Represents the item in the root of PG_DATA directory
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum PGDataItem {
    /// Represents a known PG_DATA item and its presence
    Known(DirEntry, KnownPGDataItem, PGDataItemState),

    /// Represents an unknown PG_DATA item
    Unknown(DirEntry),
}

impl Debug for PGDataItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PGDataItem::Known(dir_entry, known_pgdata_item, state) => f.write_fmt(format_args!(
                "Known({:?}, {:?}, {:?})",
                dir_entry, known_pgdata_item, state
            )),
            PGDataItem::Unknown(dir_entry) => f.write_fmt(format_args!("Unknown({:?})", dir_entry)),
        }
    }
}

/// Represents the presence of a known PG_DATA item
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum PGDataItemState {
    /// A known PG_DATA item is present
    Present,
    /// A known PG_DATA item is missing
    Missing,
}

/// Represents all known PG_DATA items
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum KnownPGDataItem {
    PGVersion,
    Base,
    CurrentLogFiles,
    Global,
    PGCommitTS,
    PGDynshmem,
    PGLogical,
    PGMultixact,
    PGNotify,
    PGReplslot,
    PGSerial,
    PGSnapshots,
    PGStat,
    PGStatTmp,
    PGSubtrans,
    PGTblspc,
    PGTwophase,
    PGWal,
    PGXact,
    PostgresqlConf,
    PostgresqlAutoConf,
    PostmasterOpts,
    PostmasterPid,
    PGHbaConf,
    PGIdentConf,
}

/// Instantiates a default implementation of [PGData]
fn pgdata(path: PathBuf) -> impl PGData {
    default_impl::PGData::new(path)
}

mod default_impl {
    use std::{
        borrow::Borrow,
        collections::BTreeSet,
        path::{Path, PathBuf},
        rc::Rc,
    };

    use crate::{
        common::fs::DirEntry,
        pgdata::{base, pg_ident_conf, pg_version, postgresql_conf},
    };

    use anyhow::anyhow;

    use super::{
        current_logfiles, global, pg_commit_ts, pg_dynshmem, pg_hba_conf, pg_logical,
        pg_multiexact, pg_notify, pg_replslot, pg_serial, pg_snapshots, pg_stat, pg_stat_tmp,
        pg_subtrans, pg_tblspc, pg_twophase, pg_wal, pg_xact, postgresql_auto_conf,
        postmaster_opts, postmaster_pid, KnownPGDataItem, PGDataItem, PGDataItemState,
    };

    // allows to query set of PGDataItems by DirEntry
    impl Borrow<DirEntry> for PGDataItem {
        fn borrow(&self) -> &DirEntry {
            match self {
                PGDataItem::Known(ref dir_entry, _, _) => dir_entry,
                PGDataItem::Unknown(ref dir_entry) => dir_entry,
            }
        }
    }

    pub struct PGData {
        path: Rc<Path>,
    }

    impl super::PGData for PGData {
        fn path(&self) -> &std::path::Path {
            &self.path
        }

        fn items(&self) -> anyhow::Result<Box<dyn Iterator<Item = PGDataItem>>> {
            let pgdata_items = read(self.path())?;
            Ok(Box::new(pgdata_items))
        }

        fn base(&self) -> Box<dyn base::Base> {
            todo!()
        }
    }

    impl PGData {
        pub fn new(path: PathBuf) -> Self {
            PGData {
                path: Rc::from(path),
            }
        }
    }

    fn read(pgdata: &Path) -> anyhow::Result<impl Iterator<Item = PGDataItem>> {
        let known_items = known_items();
        let actual_items = actual_items(pgdata)?;

        let mut set = BTreeSet::new();

        for (dir_entry, known_item) in known_items {
            set.insert(PGDataItem::Known(
                dir_entry,
                known_item,
                PGDataItemState::Missing,
            ));
        }

        for dir_entry in actual_items {
            match set.take(&dir_entry) {
                Some(PGDataItem::Known(_, known_item, _)) => {
                    set.insert(PGDataItem::Known(
                        dir_entry,
                        known_item,
                        PGDataItemState::Present,
                    ));
                }
                None => {
                    set.insert(PGDataItem::Unknown(dir_entry));
                }
                _ => (),
            }
        }

        Ok(set.into_iter())
    }

    #[rustfmt::skip]
    fn known_items() -> Vec<(DirEntry, KnownPGDataItem)> {
        vec![
            (base::dir_entry(), KnownPGDataItem::Base),
            (current_logfiles::dir_entry(), KnownPGDataItem::CurrentLogFiles),
            (global::dir_entry(), KnownPGDataItem::Global),
            (pg_commit_ts::dir_entry(), KnownPGDataItem::PGCommitTS),
            (pg_dynshmem::dir_entry(), KnownPGDataItem::PGDynshmem),
            (pg_logical::dir_entry(), KnownPGDataItem::PGLogical),
            (pg_multiexact::dir_entry(), KnownPGDataItem::PGMultixact),
            (pg_notify::dir_entry(), KnownPGDataItem::PGNotify),
            (pg_replslot::dir_entry(), KnownPGDataItem::PGReplslot),
            (pg_serial::dir_entry(), KnownPGDataItem::PGSerial),
            (pg_snapshots::dir_entry(), KnownPGDataItem::PGSnapshots),
            (pg_stat::dir_entry(), KnownPGDataItem::PGStat),
            (pg_stat_tmp::dir_entry(), KnownPGDataItem::PGStatTmp),
            (pg_subtrans::dir_entry(), KnownPGDataItem::PGSubtrans),
            (pg_tblspc::dir_entry(), KnownPGDataItem::PGTblspc),
            (pg_twophase::dir_entry(), KnownPGDataItem::PGTwophase),
            (pg_version::dir_entry(), KnownPGDataItem::PGVersion),
            (pg_wal::dir_entry(), KnownPGDataItem::PGWal),
            (pg_xact::dir_entry(), KnownPGDataItem::PGXact),
            (postgresql_conf::dir_entry(), KnownPGDataItem::PostgresqlConf),
            (postgresql_auto_conf::dir_entry(), KnownPGDataItem::PostgresqlAutoConf),
            (postmaster_opts::dir_entry(), KnownPGDataItem::PostmasterOpts),
            (postmaster_pid::dir_entry(), KnownPGDataItem::PostmasterPid),
            (pg_hba_conf::dir_entry(), KnownPGDataItem::PGHbaConf),
            (pg_ident_conf::dir_entry(), KnownPGDataItem::PGIdentConf),
        ]
    }

    fn actual_items(pgdata: &Path) -> anyhow::Result<Vec<DirEntry>> {
        let dir = std::fs::read_dir(pgdata)?;
        dir.into_iter()
            .map(|entry| entry.map_err(|err| anyhow!(err)))
            .map(|entry| entry.and_then(|fs_dir_entry| DirEntry::from(&fs_dir_entry)))
            .collect::<Result<Vec<_>, _>>()
    }

    #[cfg(test)]
    mod tests {
        use std::collections::BTreeSet;
        use std::path::PathBuf;

        use pretty_assertions::assert_eq;
        use rstest::rstest;

        use crate::pgdata::{
            base, current_logfiles, global, pg_commit_ts, pg_dynshmem, pg_hba_conf, pg_ident_conf,
            pg_logical, pg_multiexact, pg_notify, pg_replslot, pg_serial, pg_snapshots, pg_stat,
            pg_stat_tmp, pg_subtrans, pg_tblspc, pg_twophase, pg_version, pg_wal, pg_xact,
            postgresql_auto_conf, postgresql_conf, postmaster_opts, postmaster_pid,
            KnownPGDataItem, PGData, PGDataItem, PGDataItemState,
        };

        use crate::common::test_utils::fixture::*;
        #[rstest]
        fn my_test(pgdata: PathBuf) {
            let pgdata = super::PGData::new(pgdata);

            let items = pgdata.items().expect("gets pgdata.items()");

            #[rustfmt::skip]
            assert_eq!(
                items.collect::<BTreeSet<_>>(),
                BTreeSet::from([
                    PGDataItem::Known(base::dir_entry(), KnownPGDataItem::Base, PGDataItemState::Present),
                    PGDataItem::Known(current_logfiles::dir_entry(), KnownPGDataItem::CurrentLogFiles , PGDataItemState::Missing), // current_logfiles not created on pgdata setup
                    PGDataItem::Known(global::dir_entry(), KnownPGDataItem::Global, PGDataItemState::Present),
                    PGDataItem::Known(pg_commit_ts::dir_entry(), KnownPGDataItem::PGCommitTS, PGDataItemState::Present),
                    PGDataItem::Known(pg_dynshmem::dir_entry(), KnownPGDataItem::PGDynshmem, PGDataItemState::Present),
                    PGDataItem::Known(pg_hba_conf::dir_entry(), KnownPGDataItem::PGHbaConf, PGDataItemState::Present),
                    PGDataItem::Known(pg_ident_conf::dir_entry(), KnownPGDataItem::PGIdentConf, PGDataItemState::Present),
                    PGDataItem::Known(pg_logical::dir_entry(), KnownPGDataItem::PGLogical, PGDataItemState::Present),
                    PGDataItem::Known(pg_multiexact::dir_entry(), KnownPGDataItem::PGMultixact, PGDataItemState::Present),
                    PGDataItem::Known(pg_notify::dir_entry(), KnownPGDataItem::PGNotify, PGDataItemState::Present),
                    PGDataItem::Known(pg_replslot::dir_entry(), KnownPGDataItem::PGReplslot, PGDataItemState::Present),
                    PGDataItem::Known(pg_serial::dir_entry(), KnownPGDataItem::PGSerial, PGDataItemState::Present),
                    PGDataItem::Known(pg_snapshots::dir_entry(), KnownPGDataItem::PGSnapshots, PGDataItemState::Present),
                    PGDataItem::Known(pg_stat::dir_entry(), KnownPGDataItem::PGStat, PGDataItemState::Present),
                    PGDataItem::Known(pg_stat_tmp::dir_entry(), KnownPGDataItem::PGStatTmp, PGDataItemState::Present),
                    PGDataItem::Known(pg_subtrans::dir_entry(), KnownPGDataItem::PGSubtrans, PGDataItemState::Present),
                    PGDataItem::Known(pg_tblspc::dir_entry(), KnownPGDataItem::PGTblspc, PGDataItemState::Present),
                    PGDataItem::Known(pg_twophase::dir_entry(), KnownPGDataItem::PGTwophase, PGDataItemState::Present),
                    PGDataItem::Known(pg_version::dir_entry(), KnownPGDataItem::PGVersion, PGDataItemState::Present),
                    PGDataItem::Known(pg_wal::dir_entry(), KnownPGDataItem::PGWal, PGDataItemState::Present),
                    PGDataItem::Known(pg_xact::dir_entry(), KnownPGDataItem::PGXact, PGDataItemState::Present),
                    PGDataItem::Known(postgresql_auto_conf::dir_entry(), KnownPGDataItem::PostgresqlAutoConf , PGDataItemState::Present),
                    PGDataItem::Known(postgresql_conf::dir_entry(), KnownPGDataItem::PostgresqlConf , PGDataItemState::Present),
                    PGDataItem::Known(postmaster_opts::dir_entry(), KnownPGDataItem::PostmasterOpts , PGDataItemState::Present),
                    PGDataItem::Known(postmaster_pid::dir_entry(), KnownPGDataItem::PostmasterPid, PGDataItemState::Missing), // postmaster.pid not created on pgdata setup
                ])
            );
        }
    }
}
