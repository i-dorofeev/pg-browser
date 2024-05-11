use std::{
    ffi::OsString,
    fs::{read_dir, DirEntry},
    path::Path,
};

use anyhow::{bail, Context, Error, Result};

use crate::common::{FileType, PgOid, SimpleDirEntry};

pub fn base_dir_reader(path: &Path) -> impl BaseDirReader + '_ {
    DefaultBaseDirReader { path }
}

pub trait BaseDirReader {
    fn read_base_dir(&self) -> Result<BaseDir, Error>;
}

#[derive(Debug, PartialEq)]
pub struct BaseDir(pub Vec<BaseDirItem>);

impl BaseDir {
    pub fn items(&self) -> &[BaseDirItem] {
        let BaseDir(items) = self;
        &items[..]
    }
}

#[derive(Debug)]
pub enum BaseDirItem {
    DatabaseDir(DatabaseDir),
    UnknownEntry(SimpleDirEntry),
    Error(anyhow::Error),
}

impl PartialEq for BaseDirItem {
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

impl BaseDirItem {
    pub fn name(&self) -> anyhow::Result<OsString> {
        match self {
            BaseDirItem::DatabaseDir(DatabaseDir { oid: _, db_name }) => {
                OsString::try_from(db_name).context("")
            }
            BaseDirItem::UnknownEntry(SimpleDirEntry {
                name,
                entry_type: _,
            }) => Ok(name.to_owned()),
            BaseDirItem::Error(_) => bail!("Error"),
        }
    }

    pub fn database_dir(pg_oid: u32, db_name: &'static str) -> BaseDirItem {
        BaseDirItem::DatabaseDir(DatabaseDir {
            oid: PgOid(pg_oid),
            db_name: db_name.into(),
        })
    }

    pub fn unknown_file(name: &'static str) -> BaseDirItem {
        BaseDirItem::UnknownEntry(SimpleDirEntry {
            name: name.into(),
            entry_type: FileType::File,
        })
    }

    pub fn unknown_dir(name: &'static str) -> BaseDirItem {
        BaseDirItem::UnknownEntry(SimpleDirEntry {
            name: name.into(),
            entry_type: FileType::Dir,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct DatabaseDir {
    pub oid: PgOid,
    pub db_name: String,
}

impl DatabaseDir {
    pub fn from(dir_entry: &DirEntry) -> Result<Option<DatabaseDir>> {
        let entry_name = dir_entry.file_name().to_string_lossy().to_string();
        let entry_type = dir_entry
            .file_type()
            .with_context(|| format!("DirEntry(file_name = {}).file_type()", entry_name))?;

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
}

struct DefaultBaseDirReader<'a> {
    path: &'a Path,
}

impl<'a> BaseDirReader for DefaultBaseDirReader<'a> {
    fn read_base_dir(&self) -> Result<BaseDir, anyhow::Error> {
        let read_dir = read_dir(self.path).with_context(|| format!("Reading {:?}", self.path))?;
        let base_dir_items: Vec<BaseDirItem> = read_dir
            .map(|maybe_dir_entry| {
                maybe_dir_entry
                    .map_err(Error::new)
                    .map_or_else(BaseDirItem::Error, |dir_entry| to_base_dir_item(&dir_entry))
            })
            .collect();
        Ok(BaseDir(base_dir_items))
    }
}

fn to_base_dir_item(dir_entry: &DirEntry) -> BaseDirItem {
    match DatabaseDir::from(dir_entry) {
        Ok(Some(database_dir)) => BaseDirItem::DatabaseDir(database_dir),
        Ok(None) => SimpleDirEntry::from(dir_entry)
            .map_or_else(BaseDirItem::Error, BaseDirItem::UnknownEntry),
        Err(err) => BaseDirItem::Error(err),
    }
}
