use std::ffi::OsString;
use std::fs::DirEntry as StdDirEntry;
use std::{borrow::Cow, ffi::OsStr, path::Path};

use crate::common::fs::DirEntry;
use crate::common::{self, PgOid};
use anyhow::bail;
use anyhow::Context;

use self::db_dir::DbDir;

pub mod db_dir;

pub trait Base {
    fn path(&self) -> &Path;
    fn items(
        &self,
    ) -> anyhow::Result<
        impl IntoIterator<Item = BaseDirItem, IntoIter = impl Iterator<Item = BaseDirItem>>,
    >;
    fn db_dir<'a>(&self, oid: PgOid) -> anyhow::Result<impl DbDir + 'a>;
}

#[derive(Debug)]
pub enum BaseDirItem<'a> {
    DatabaseDir(DatabaseDir<'a>),
    UnknownEntry(DirEntry<'a>),
    Error(anyhow::Error),
}

impl BaseDirItem<'_> {
    pub fn name(&self) -> anyhow::Result<Cow<'_, OsStr>> {
        match self {
            BaseDirItem::DatabaseDir(DatabaseDir {
                oid: PgOid(oid),
                db_name: _,
            }) => Ok(<OsString>::from(oid.to_string()).into()),
            BaseDirItem::UnknownEntry(DirEntry {
                name,
                entry_type: _,
            }) => Ok(Cow::Borrowed(name)),
            BaseDirItem::Error(_) => bail!("Error"),
        }
    }

    pub fn database_dir(pg_oid: u32, db_name: &str) -> BaseDirItem {
        BaseDirItem::DatabaseDir(DatabaseDir {
            oid: PgOid(pg_oid),
            db_name: db_name.into(),
        })
    }

    pub fn unknown_file(name: &str) -> BaseDirItem<'static> {
        BaseDirItem::UnknownEntry(DirEntry::file(name))
    }

    pub fn unknown_dir(name: &str) -> BaseDirItem<'static> {
        BaseDirItem::UnknownEntry(DirEntry::dir(name))
    }
}

impl PartialEq for BaseDirItem<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (BaseDirItem::DatabaseDir(dir1), BaseDirItem::DatabaseDir(dir2)) => dir1 == dir2,
            (BaseDirItem::UnknownEntry(entry1), BaseDirItem::UnknownEntry(entry2)) => {
                entry1 == entry2
            }
            (BaseDirItem::Error(_), BaseDirItem::Error(_)) => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DatabaseDir<'a> {
    oid: PgOid,
    db_name: Cow<'a, str>,
}

impl DatabaseDir<'_> {
    pub fn from(dir_entry: &StdDirEntry) -> anyhow::Result<Option<DatabaseDir<'static>>> {
        let entry_name = dir_entry.file_name();
        let entry_type = dir_entry
            .file_type()
            .with_context(|| format!("DirEntry(file_name = {:?}).file_type()", entry_name))?;

        if !entry_type.is_dir() {
            return Ok(None);
        }

        match PgOid::try_parse(&entry_name) {
            Some(oid) => Ok(Some(DatabaseDir {
                oid,
                db_name: "TODO: database name".into(),
            })),
            None => Ok(None),
        }
    }

    pub fn dir_name(&self) -> String {
        let DatabaseDir {
            oid: PgOid(oid), ..
        } = self;
        format!("{}", oid)
    }

    pub fn db_name(&self) -> &str {
        &self.db_name
    }
}

pub fn dir_entry() -> common::fs::DirEntry<'static> {
    common::fs::DirEntry::dir("base")
}

/// Instantiates a default implementation of [Base]
pub fn base(pgdata_path: &Path) -> impl Base {
    default_impl::Base::new(pgdata_path)
}

mod default_impl {
    use std::fs::DirEntry as StdDirEntry;
    use std::iter::empty;
    use std::{
        fs::read_dir,
        path::{Path, PathBuf},
        rc::Rc,
    };

    use anyhow::{Context, Error};

    use crate::common::fs::DirEntry;
    use crate::common::PgOid;

    use super::db_dir::DbDir;
    use super::{BaseDirItem, DatabaseDir};

    pub struct Base {
        path: Rc<PathBuf>,
    }

    impl super::Base for Base {
        fn path(&self) -> &Path {
            &self.path
        }

        fn items(
            &self,
        ) -> anyhow::Result<
            impl IntoIterator<
                Item = super::BaseDirItem,
                IntoIter = impl Iterator<Item = super::BaseDirItem>,
            >,
        > {
            let read_dir =
                read_dir(self.path.as_ref()).with_context(|| format!("Reading {:?}", self.path))?;
            let items = read_dir.map(|maybe_dir_entry| {
                maybe_dir_entry
                    .map_err(Error::new)
                    .map_or_else(BaseDirItem::Error, |dir_entry| to_base_dir_item(&dir_entry))
            });
            Ok(items)
        }

        fn db_dir<'a>(&self, _oid: PgOid) -> anyhow::Result<impl DbDir + 'a> {
            Ok(StubBaseDir {})
        }
    }

    struct StubBaseDir;
    impl DbDir for StubBaseDir {
        fn items(
            &self,
        ) -> anyhow::Result<
            impl IntoIterator<
                Item = super::db_dir::DbDirItem,
                IntoIter = impl Iterator<Item = super::db_dir::DbDirItem>,
            >,
        > {
            Ok(empty())
        }
    }

    impl Base {
        pub fn new(pgdata_path: &Path) -> Self {
            Base {
                path: pgdata_path.join("base").into(),
            }
        }
    }

    fn to_base_dir_item<'a>(dir_entry: &StdDirEntry) -> BaseDirItem<'a> {
        match DatabaseDir::from(dir_entry) {
            Ok(Some(database_dir)) => BaseDirItem::DatabaseDir(database_dir),
            Ok(None) => {
                DirEntry::from(dir_entry).map_or_else(BaseDirItem::Error, BaseDirItem::UnknownEntry)
            }
            Err(err) => BaseDirItem::Error(err),
        }
    }

    #[cfg(test)]
    pub mod tests {
        use std::path::PathBuf;

        use super::BaseDirItem;
        use crate::{common::test_utils::fixture::*, pgdata::base::Base};
        use pretty_assertions::assert_eq;
        use rstest::rstest;

        #[rstest]
        fn reads_base_dir(pgdata: PathBuf) {
            // given
            let base = super::Base::new(&pgdata);

            // when
            let items_iter = base.items().unwrap().into_iter();
            let mut items = items_iter.collect::<Vec<BaseDirItem>>();
            items.sort_by_key(|base_dir_item| {
                base_dir_item.name().expect("BaseDirItem").into_owned()
            });

            // then
            assert_eq!(
                items,
                vec![
                    BaseDirItem::database_dir(1, "TODO: database name"),
                    BaseDirItem::database_dir(4, "TODO: database name"),
                    BaseDirItem::database_dir(5, "TODO: database name")
                ]
            )
        }
    }
}

#[cfg(test)]
pub mod test_stubs {
    use std::path::Path;

    use crate::common::PgOid;

    use super::{
        db_dir::{test_stubs::StubDbDir, DbDir},
        Base,
    };

    pub struct StubBase;
    impl Base for StubBase {
        fn path(&self) -> &Path {
            todo!()
        }

        fn items(
            &self,
        ) -> anyhow::Result<
            impl IntoIterator<
                Item = super::BaseDirItem,
                IntoIter = impl Iterator<Item = super::BaseDirItem>,
            >,
        > {
            Ok(std::iter::empty())
        }

        fn db_dir<'a>(&self, _oid: PgOid) -> anyhow::Result<impl DbDir + 'a> {
            Ok(StubDbDir {})
        }
    }
}
