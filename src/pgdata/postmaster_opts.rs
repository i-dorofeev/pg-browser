use crate::common::fs::DirEntry;

pub trait PostmasterOpts {}

pub fn dir_entry() -> DirEntry {
    DirEntry::file("postmaster.opts")
}
