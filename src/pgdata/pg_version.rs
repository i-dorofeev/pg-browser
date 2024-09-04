use crate::common::fs::DirEntry;

#[allow(dead_code)]
pub trait PGVersion {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::file("PG_VERSION")
}
