use crate::common::fs::DirEntry;

pub trait Base {}

pub fn dir_entry() -> DirEntry {
    DirEntry::dir("base")
}
