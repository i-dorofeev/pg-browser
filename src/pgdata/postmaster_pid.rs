use crate::common::fs::DirEntry;

#[allow(dead_code)]
pub trait PostmasterPid {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::file("postmaster.pid")
}
