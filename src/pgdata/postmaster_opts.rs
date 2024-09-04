use crate::common::fs::DirEntry;

#[allow(dead_code)]
pub trait PostmasterOpts {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::file("postmaster.opts")
}
