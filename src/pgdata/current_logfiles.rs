use crate::common::fs::DirEntry;

pub trait CurrentLogFiles {}

pub fn dir_entry() -> DirEntry {
    DirEntry::file("current_logfiles")
}
