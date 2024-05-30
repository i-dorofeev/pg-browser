use anyhow::{anyhow, Context, Result};
use std::{
    ffi::{OsStr, OsString},
    fmt::Debug,
    rc::Rc,
};

#[derive(PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct DirEntry {
    name: Rc<OsStr>,
    entry_type: FileType,
}

impl Debug for DirEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}: {:?}", self.entry_type, self.name))
    }
}

impl Clone for DirEntry {
    fn clone(&self) -> Self {
        DirEntry {
            name: self.name.clone(),
            entry_type: self.entry_type,
        }
    }
}

impl DirEntry {
    // TODO: tests for DirEntry::from
    pub fn from(dir_entry: &std::fs::DirEntry) -> Result<DirEntry> {
        let fs_file_type = dir_entry
            .file_type()
            .with_context(|| format!("DirEntry.from({:?})", dir_entry.path()))?;
        let file_type = FileType::from(fs_file_type)
            .with_context(|| format!("DirEntry.from({:?})", dir_entry.path()))?;

        Ok(DirEntry {
            name: Rc::from(dir_entry.file_name().as_os_str()),
            entry_type: file_type,
        })
    }

    pub fn file(name: &'static str) -> DirEntry {
        DirEntry {
            name: Rc::from(OsString::from(name).as_os_str()),
            entry_type: FileType::File,
        }
    }

    pub fn dir(name: &'static str) -> DirEntry {
        DirEntry {
            name: Rc::from(OsString::from(name).as_os_str()),
            entry_type: FileType::Dir,
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
