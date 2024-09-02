use crate::common::fs::DirEntry;

pub trait PostmasterOpts {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::file("postmaster.opts")
}
