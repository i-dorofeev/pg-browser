pub mod base;
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

use std::{fmt::Debug, path::Path, rc::Rc};

use strum_macros::EnumIter;

use crate::common::fs::DirEntry;

use self::base::Base;

/// Represents the PG_DATA directory
pub trait PGData {
    /// Returns the actual path of the directory represented by this trait
    fn path(&self) -> &Path;

    /// Lists all expected and unexpected items of PG_DATA
    fn list_items(
        &self,
    ) -> anyhow::Result<
        impl IntoIterator<Item = PGDataItem, IntoIter = impl Iterator<Item = PGDataItem>>,
    >;

    /// Represents PG_DATA/base directory
    fn items(&self) -> impl PGDataItems;
}

pub trait PGDataItems {
    /// Represents PG_DATA/base directory
    fn base<'a, 'b>(&'a self) -> impl Base + 'b;
}

/// Represents the item in the root of PG_DATA directory
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum PGDataItem<'a> {
    /// Represents a known PG_DATA item and its presence
    Known(DirEntry<'a>, KnownPGDataItem, PGDataItemState),

    /// Represents an unknown PG_DATA item
    Unknown(DirEntry<'a>),
}

impl Debug for PGDataItem<'_> {
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

impl PGDataItem<'_> {
    pub fn known_present_dir(
        dir_name: &'static str,
        known_pgdata_item: KnownPGDataItem,
    ) -> PGDataItem {
        PGDataItem::Known(
            DirEntry::dir(dir_name),
            known_pgdata_item,
            PGDataItemState::Present,
        )
    }

    pub fn known_present_file(
        file_name: &'static str,
        known_pgdata_item: KnownPGDataItem,
    ) -> PGDataItem {
        PGDataItem::Known(
            DirEntry::file(file_name),
            known_pgdata_item,
            PGDataItemState::Present,
        )
    }
}

/// Represents the presence of a known PG_DATA item
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone)]
pub enum PGDataItemState {
    /// A known PG_DATA item is present
    Present,
    /// A known PG_DATA item is missing
    Missing,
}

/// Represents all known PG_DATA items
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, EnumIter)]
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
pub fn pgdata(path: Rc<Path>) -> impl PGData {
    default_impl::PGData::new(path)
}

mod default_impl {
    use std::{borrow::Borrow, collections::BTreeSet, path::Path, rc::Rc};

    use crate::{
        common::fs::DirEntry,
        pgdata::{base, pg_ident_conf, pg_version, postgresql_conf},
    };

    use anyhow::anyhow;

    use super::{
        base::Base, current_logfiles, global, pg_commit_ts, pg_dynshmem, pg_hba_conf, pg_logical,
        pg_multiexact, pg_notify, pg_replslot, pg_serial, pg_snapshots, pg_stat, pg_stat_tmp,
        pg_subtrans, pg_tblspc, pg_twophase, pg_wal, pg_xact, postgresql_auto_conf,
        postmaster_opts, postmaster_pid, KnownPGDataItem, PGDataItem, PGDataItemState,
    };

    // allows to query set of PGDataItems by DirEntry
    impl<'a> Borrow<DirEntry<'a>> for PGDataItem<'a> {
        fn borrow(&self) -> &DirEntry<'a> {
            match self {
                PGDataItem::Known(ref dir_entry, _, _) => dir_entry,
                PGDataItem::Unknown(ref dir_entry) => dir_entry,
            }
        }
    }

    pub struct PGData {
        path: Rc<Path>,
    }

    struct PGDataItems<'a> {
        pgdata: &'a PGData,
    }

    impl super::PGDataItems for PGDataItems<'_> {
        fn base<'a, 'b>(&'a self) -> impl Base + 'b {
            base::base(&self.pgdata.path)
        }
    }

    impl super::PGData for PGData {
        fn path(&self) -> &Path {
            self.path.borrow()
        }

        fn list_items(
            &self,
        ) -> anyhow::Result<
            impl IntoIterator<Item = PGDataItem, IntoIter = impl Iterator<Item = PGDataItem>>,
        > {
            let pgdata_items = read(self.path.as_ref())?;
            Ok(pgdata_items.into_iter())
        }

        fn items(&self) -> impl super::PGDataItems {
            PGDataItems { pgdata: self }
        }
    }

    impl PGData {
        pub fn new(path: Rc<Path>) -> PGData {
            PGData { path }
        }
    }

    fn read(pgdata: &Path) -> anyhow::Result<BTreeSet<PGDataItem>> {
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

        Ok(set)
    }

    #[rustfmt::skip]
    fn known_items() -> Vec<(DirEntry<'static>, KnownPGDataItem)> {
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

    fn actual_items(pgdata: &Path) -> anyhow::Result<Vec<DirEntry<'_>>> {
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
        fn reads_root_dir(pgdata: PathBuf) {
            let pgdata = super::PGData::new(pgdata.into());

            let items = pgdata.list_items().expect("gets pgdata.items()");

            #[rustfmt::skip]
            assert_eq!(
                items.into_iter().collect::<BTreeSet<_>>(),
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

#[cfg(test)]
pub mod test_stubs {
    use super::{
        base::{test_stubs::StubBase, Base},
        PGDataItems,
    };

    pub struct StubPGDataItems;
    impl PGDataItems for StubPGDataItems {
        fn base<'a, 'b>(&'a self) -> impl Base + 'b {
            StubBase {}
        }
    }
}
