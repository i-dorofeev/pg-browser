use crate::common::fs::DirEntry;

pub trait PGReplslot {}

pub fn dir_entry() -> DirEntry {
    DirEntry::dir("pg_replslot")
}
