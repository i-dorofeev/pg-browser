use std::{
    ffi::OsString,
    fs::{DirEntry, FileType as FsFileType},
};

use anyhow::{anyhow, Context, Result};

#[derive(Debug, PartialEq)]
pub struct PgOid(pub u32);

impl PgOid {
    // TODO: tests for PgOid::try_parse
    pub fn try_parse(s: &str) -> Option<Self> {
        match s.parse::<u32>() {
            Ok(oid) => Some(PgOid(oid)),
            Err(_) => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FileType {
    Dir,
    File,
}

impl FileType {
    pub fn from(file_type: FsFileType) -> Result<FileType> {
        match file_type {
            _ if file_type.is_dir() => Ok(FileType::Dir),
            _ if file_type.is_file() => Ok(FileType::File),
            _ => Err(anyhow!("Unknown FileType {:?}", file_type)),
        }
    }
}

pub fn render_file_type(file_type: &FileType) -> String {
    match file_type {
        FileType::Dir => "D",
        FileType::File => "F",
    }
    .to_string()
}

#[derive(Debug, PartialEq)]
pub struct SimpleDirEntry {
    pub name: OsString,
    pub entry_type: FileType,
}

impl SimpleDirEntry {
    // TODO: tests for SimpleDirEntry::from
    pub fn from(dir_entry: &DirEntry) -> Result<SimpleDirEntry> {
        let fs_file_type = dir_entry
            .file_type()
            .with_context(|| format!("SimpleDirEntry.from({:?})", dir_entry.path()))?;
        let file_type = FileType::from(fs_file_type)
            .with_context(|| format!("SimpleDirEntry.from({:?})", dir_entry.path()))?;
        Ok(SimpleDirEntry {
            name: dir_entry.file_name(),
            entry_type: file_type,
        })
    }
}
