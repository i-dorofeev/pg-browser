use std::{
    fs::{read_dir, DirEntry},
    path::PathBuf,
};

use anyhow::{anyhow, Context, Error, Result};

use crate::common::result_option::ResultOption;
use crate::common::{PgOid, SimpleDirEntry};

/*******************/
/* Data structures */
/*******************/

#[derive(Debug)]
pub struct DbDir(pub Vec<DbDirItem>);

#[derive(Debug)]
pub enum DbDirItem {
    ForkSegmentFile(ForkSegmentFile),
    FileNodeMapFile,
    PgVersionFile,
    UnknownEntry(SimpleDirEntry),
    Error(anyhow::Error),
}

#[derive(Debug, PartialEq)]
pub struct ForkSegmentFile {
    pub fork_type: ForkType,
    pub oid: PgOid,
    pub segment_id: u16,
}

impl ForkSegmentFile {
    fn create(oid: u32, fork_type: ForkType, segment_id: u16) -> Self {
        ForkSegmentFile {
            fork_type,
            oid: PgOid(oid),
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

/**************/
/* Module API */
/**************/

pub fn db_dir_reader(path: PathBuf) -> impl DbDirReader {
    DefaultDbDirReader { path }
}

pub trait DbDirReader {
    fn read_db_dir(&self) -> Result<DbDir, Error>;
}

/*************************/
/* Module implementation */
/*************************/

struct DefaultDbDirReader {
    path: PathBuf,
}

impl DbDirReader for DefaultDbDirReader {
    fn read_db_dir(&self) -> Result<DbDir, Error> {
        let dir = read_dir(&self.path).with_context(|| format!("Reading {:?}", self.path))?;
        let db_dir_items: Vec<DbDirItem> = dir
            .map(|maybe_dir_entry| {
                maybe_dir_entry
                    .map_err(Error::new)
                    .map_or_else(DbDirItem::Error, |dir_entry| to_db_dir_item(&dir_entry))
            })
            .collect();
        Ok(DbDir(db_dir_items))
    }
}

fn to_db_dir_item(dir_entry: &DirEntry) -> DbDirItem {
    try_to_db_dir_item(dir_entry).unwrap_or_else(DbDirItem::Error)
}

fn try_to_db_dir_item(dir_entry: &DirEntry) -> Result<DbDirItem> {
    fork_segment_file(dir_entry)
        .or_if_empty(|| file_node_map_file(dir_entry))
        .or_if_empty(|| pg_version_file(dir_entry))
        .otherwise(|| SimpleDirEntry::from(dir_entry).map(DbDirItem::UnknownEntry))
}

fn fork_segment_file(dir_entry: &DirEntry) -> Result<Option<DbDirItem>> {
    let file_name = dir_entry
        .file_name()
        .into_string()
        .map_err(|os_str| anyhow!("Cannot convert OsStr:{:?} into String", os_str))?;

    Ok(ForkSegmentFile::try_parse(&file_name).map(DbDirItem::ForkSegmentFile))
}

fn file_node_map_file(dir_entry: &DirEntry) -> Result<Option<DbDirItem>> {
    todo!()
}

fn pg_version_file(dir_entry: &DirEntry) -> Result<Option<DbDirItem>> {
    todo!()
}

mod fork_segment_file_impl {
    use once_cell::sync::Lazy;
    use regex::Regex;

    use crate::common::PgOid;

    use super::{ForkSegmentFile, ForkType};

    static FORK_SEGMENT_FILE_REGEX: Lazy<Regex> =
        regex_static::lazy_regex!(r"^([0-9]{1,10})(_(fsm|vm))?(\.([0-9]*))?$");

    impl ForkSegmentFile {
        pub fn try_parse(file_name: &str) -> Option<ForkSegmentFile> {
            match FORK_SEGMENT_FILE_REGEX.captures(&file_name) {
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
                        .map_or(None, Option::Some);

                    match (oid, fork_type, segment_id) {
                        (Some(oid), Some(fork_type), Some(segment_id)) => Some(ForkSegmentFile {
                            fork_type,
                            oid,
                            segment_id,
                        }),
                        _ => None,
                    }
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
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
        fn parses_fork_segment_file(
            #[case] file_name: &str,
            #[case] expected: (u32, ForkType, u16),
        ) {
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
}

impl ForkType {
    fn try_parse(s: Option<&str>) -> Option<ForkType> {
        match s {
            None => Some(ForkType::Main),
            Some("fsm") => Some(ForkType::FreeSpaceMap),
            Some("vm") => Some(ForkType::VisibilityMap),
            _ => None,
        }
    }
}
