use crate::common::fs::DirEntry;

pub trait PostmasterPid {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::file("postmaster.pid")
}
