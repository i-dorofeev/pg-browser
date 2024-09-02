use crate::common::fs::DirEntry;

pub trait PGVersion {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::file("PG_VERSION")
}
