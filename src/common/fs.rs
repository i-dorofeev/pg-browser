use anyhow::{anyhow, Context, Result};
use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    fmt::Debug,
};

use std::fs::DirEntry as StdDirEntry;

#[derive(PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct DirEntry<'a> {
    pub name: Cow<'a, OsStr>,
    pub entry_type: FileType,
}

impl Debug for DirEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}: {:?}", self.entry_type, self.name))
    }
}

impl Clone for DirEntry<'_> {
    fn clone(&self) -> Self {
        DirEntry {
            name: self.name.clone(),
            entry_type: self.entry_type,
        }
    }
}

impl DirEntry<'_> {
    // TODO: tests for DirEntry::from
    pub fn from(dir_entry: &StdDirEntry) -> Result<DirEntry<'static>> {
        let fs_file_type = dir_entry
            .file_type()
            .with_context(|| format!("DirEntry.from({:?})", dir_entry.path()))?;
        let file_type = FileType::from(fs_file_type)
            .with_context(|| format!("DirEntry.from({:?})", dir_entry.path()))?;

        Ok(DirEntry {
            name: dir_entry.file_name().into(),
            entry_type: file_type,
        })
    }

    pub fn file(name: &str) -> DirEntry<'static> {
        DirEntry::entry(FileType::File, name)
    }

    pub fn dir(name: &str) -> DirEntry<'static> {
        DirEntry::entry(FileType::Dir, name)
    }

    pub fn entry(entry_type: FileType, name: &str) -> DirEntry<'static> {
        DirEntry {
            name: OsString::from(name).into(),
            entry_type,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub enum FileType {
    Dir,
    File,
}

impl FileType {
    pub fn from(file_type: std::fs::FileType) -> Result<FileType> {
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