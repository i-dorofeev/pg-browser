use crate::common::fs::DirEntry;

#[allow(dead_code)]
pub trait PGReplslot {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::dir("pg_replslot")
}
