use std::{
    ffi::OsString,
    fs::{read_dir, DirEntry},
    path::Path,
};

use anyhow::{Context, Error};

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
    Unknown { file_name: OsString },
    Error(anyhow::Error),
}

impl PartialEq for BaseDirItem {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (BaseDirItem::DatabaseDir(dir1), BaseDirItem::DatabaseDir(dir2)) => dir1 == dir2,
            (BaseDirItem::Unknown { file_name: fn1 }, BaseDirItem::Unknown { file_name: fn2 }) => {
                fn1 == fn2
            }
            (BaseDirItem::Error(_), BaseDirItem::Error(_)) => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DatabaseDir {
    pub name: OsString,
}

#[derive(Debug, PartialEq)]
pub struct DatabaseOID(String);

impl DatabaseDir {
    #[allow(dead_code)]
    fn oid(&self) -> DatabaseOID {
        DatabaseOID(self.name.to_string_lossy().to_string())
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
                    .map_or_else(BaseDirItem::Error, to_base_dir_item)
            })
            .collect();
        Ok(BaseDir(base_dir_items))
    }
}

fn to_base_dir_item(dir_entry: DirEntry) -> BaseDirItem {
    match dir_entry.file_type() {
        Ok(file_type) if file_type.is_dir() => BaseDirItem::DatabaseDir(DatabaseDir {
            name: dir_entry.file_name(),
        }),
        Ok(_) => BaseDirItem::Unknown {
            file_name: dir_entry.file_name(),
        },
        Err(err) => BaseDirItem::Error(Error::new(err)),
    }
}
