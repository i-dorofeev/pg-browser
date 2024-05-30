use crate::common::fs::DirEntry;

pub trait PostmasterPid {}

pub fn dir_entry() -> DirEntry {
    DirEntry::file("postmaster.pid")
}
