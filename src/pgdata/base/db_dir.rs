use anyhow::anyhow;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::common;
use crate::common::PgOid;
use std::fs::DirEntry as StdDirEntry;

use anyhow::Result;

use crate::common::{fs::DirEntry, result_option::ResultOption};

pub trait DbDir {
    fn items(
        &self,
    ) -> anyhow::Result<
        impl IntoIterator<Item = DbDirItem, IntoIter = impl Iterator<Item = DbDirItem>>,
    >;
}

#[derive(Debug, PartialEq)]
pub enum DbDirItem<'a> {
    ForkSegmentFile(ForkSegmentFile),
    FileNodeMapFile,
    PgVersionFile,
    UnknownEntry(DirEntry<'a>),
    Error(common::Error),
}

impl DbDirItem<'_> {
    pub fn from_anyhow_error(error: anyhow::Error) -> DbDirItem<'static> {
        DbDirItem::Error(error.into())
    }

    pub fn from_io_error(error: std::io::Error) -> DbDirItem<'static> {
        Self::from_anyhow_error(anyhow!(error))
    }

    pub fn from_dir_entry(dir_entry: &StdDirEntry) -> DbDirItem<'static> {
        DbDirItem::try_from_dir_entry(dir_entry).unwrap_or_else(DbDirItem::from_anyhow_error)
    }

    fn try_from_dir_entry(std_dir_entry: &StdDirEntry) -> Result<DbDirItem<'static>> {
        let dir_entry_name = std_dir_entry.file_name();
        let dir_entry_name_lossy = dir_entry_name.to_string_lossy();
        let name_str = dir_entry_name_lossy.as_ref();

        DbDirItem::fork_segment_file(name_str)
            .or_if_empty(|| DbDirItem::file_node_map_file(name_str))
            .or_if_empty(|| DbDirItem::pg_version_file(name_str))
            .otherwise(|| DirEntry::from(std_dir_entry).map(DbDirItem::UnknownEntry))
    }

    fn fork_segment_file(dir_entry_name: &str) -> Result<Option<DbDirItem<'static>>> {
        Ok(ForkSegmentFile::try_parse(dir_entry_name).map(DbDirItem::ForkSegmentFile))
    }

    fn file_node_map_file(dir_entry_name: &str) -> Result<Option<DbDirItem<'static>>> {
        match dir_entry_name {
            "pg_filenode.map" => Ok(Some(DbDirItem::FileNodeMapFile)),
            _ => Ok(None),
        }
    }

    fn pg_version_file(dir_entry_name: &str) -> Result<Option<DbDirItem<'static>>> {
        match dir_entry_name {
            "PG_VERSION" => Ok(Some(DbDirItem::PgVersionFile)),
            _ => Ok(None),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ForkSegmentFile {
    fork_type: ForkType,
    oid: PgOid,
    segment_id: u16,
}

static FORK_SEGMENT_FILE_REGEX: Lazy<Regex> =
    regex_static::lazy_regex!(r"^([0-9]{1,10})(_(fsm|vm))?(\.([0-9]*))?$");

impl ForkSegmentFile {
    pub fn try_parse(file_name: &str) -> Option<ForkSegmentFile> {
        match FORK_SEGMENT_FILE_REGEX.captures(file_name) {
            None => None,
            Some(captures) => {
                let oid = captures
                    .get(1)
                    .map(|m| m.as_str())
                    .and_then(PgOid::try_parse);
                let fork_type = ForkType::try_parse(captures.get(3).map(|m| m.as_str()));
                let segment_id = captures
                    .get(5)
                    .map_or("0", |m| m.as_str())
                    .parse::<u16>()
                    .ok();

                match (oid, fork_type, segment_id) {
                    (Some(oid), Some(fork_type), Some(segment_id)) => {
                        Some(ForkSegmentFile::create(oid, fork_type, segment_id))
                    }
                    _ => None,
                }
            }
        }
    }

    pub fn create<T>(oid: T, fork_type: ForkType, segment_id: u16) -> Self
    where
        T: Into<PgOid>,
    {
        ForkSegmentFile {
            fork_type,
            oid: oid.into(),
            segment_id,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ForkType {
    Main,
    FreeSpaceMap,
    VisibilityMap,
}

impl ForkType {
    pub fn try_parse(s: Option<&str>) -> Option<ForkType> {
        match s {
            None => Some(ForkType::Main),
            Some("fsm") => Some(ForkType::FreeSpaceMap),
            Some("vm") => Some(ForkType::VisibilityMap),
            _ => None,
        }
    }
}

mod default_impl {
    use anyhow::Context;
    use std::{fs::read_dir, path::PathBuf};

    use super::DbDirItem;

    #[allow(dead_code)]
    struct DbDir {
        path: PathBuf,
    }

    impl super::DbDir for DbDir {
        fn items(
            &self,
        ) -> anyhow::Result<
            impl IntoIterator<Item = DbDirItem, IntoIter = impl Iterator<Item = DbDirItem>>,
        > {
            let dir = read_dir(&self.path).with_context(|| format!("Reading {:?}", self.path))?;
            let db_dir_item = dir.map(|maybe_dir_entry| match maybe_dir_entry {
                Ok(dir_entry) => DbDirItem::from_dir_entry(&dir_entry),
                Err(err) => DbDirItem::from_io_error(err),
            });
            Ok(db_dir_item)
        }
    }
}

#[cfg(test)]
mod db_dir_tests {
    use anyhow::Result;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::{common::stringify, pgdata::base::db_dir::DbDirItem};

    #[rstest]
    #[case("pg_filenode.map", Ok(Some(DbDirItem::FileNodeMapFile)))]
    #[case("some_other_file", Ok(None))]
    fn parses_file_node_map_file(
        #[case] file_name: &str,
        #[case] expected: Result<Option<DbDirItem<'static>>>,
    ) {
        // when

        use crate::pgdata::base::db_dir::DbDirItem;
        let result = DbDirItem::file_node_map_file(file_name);

        // then
        assert_eq!(result.map_err(stringify), expected.map_err(stringify));
    }

    #[rstest]
    #[case("PG_VERSION", Ok(Some(DbDirItem::PgVersionFile)))]
    #[case("some_other_file", Ok(None))]
    fn parses_pg_version_file(
        #[case] file_name: &str,
        #[case] expected: Result<Option<DbDirItem>>,
    ) {
        // when
        let result = DbDirItem::pg_version_file(file_name);

        // then
        assert_eq!(result.map_err(stringify), expected.map_err(stringify));
    }
}

#[cfg(test)]
mod fork_type_tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::pgdata::base::db_dir::ForkType;

    #[rstest]
    #[case(None, Some(ForkType::Main))]
    #[case(Some("fsm"), Some(ForkType::FreeSpaceMap))]
    #[case(Some("vm"), Some(ForkType::VisibilityMap))]
    #[case(Some("arb_string"), None)]
    fn parses_fork_type(#[case] s: Option<&str>, #[case] expected: Option<ForkType>) {
        // when

        use crate::pgdata::base::db_dir::ForkType;
        let result = ForkType::try_parse(s);

        // then
        assert_eq!(result, expected);
    }
}

#[cfg(test)]
mod fork_segment_file_tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{ForkSegmentFile, ForkType};

    #[rstest]
    #[case("12345", (12345, ForkType::Main, 0))]
    #[case("12345.1", (12345, ForkType::Main, 1))]
    #[case("12345_fsm", (12345, ForkType::FreeSpaceMap, 0))]
    #[case("12345_fsm.2", (12345, ForkType::FreeSpaceMap, 2))]
    #[case("12345_vm", (12345, ForkType::VisibilityMap, 0))]
    #[case("12345_vm.3", (12345, ForkType::VisibilityMap, 3))]
    fn parses_fork_segment_file(#[case] file_name: &str, #[case] expected: (u32, ForkType, u16)) {
        // given
        let (oid, fork_type, segment_id) = expected;
        let expected = ForkSegmentFile::create(oid, fork_type, segment_id);

        // when
        let parsed = ForkSegmentFile::try_parse(file_name);

        // then
        assert_eq!(parsed, Some(expected));
    }

    #[rstest]
    #[case("pg_filenode.map")]
    #[case("PG_VERSION")]
    #[case("12340_qqq")]
    #[case("12341_qqq.1")]
    #[case("12342_fsm.q")]
    #[case("12343_vm.q")]
    fn does_not_parse_as_fork_segment_file(#[case] file_name: &str) {
        // when
        let parsed = ForkSegmentFile::try_parse(file_name);

        // then
        assert_eq!(parsed, None);
    }
}

#[cfg(test)]
pub mod test_stubs {
    use std::iter::empty;

    use super::DbDir;

    pub struct StubDbDir;
    impl DbDir for StubDbDir {
        fn items(
            &self,
        ) -> anyhow::Result<
            impl IntoIterator<
                Item = super::DbDirItem,
                IntoIter = impl Iterator<Item = super::DbDirItem>,
            >,
        > {
            Ok(empty())
        }
    }
}
