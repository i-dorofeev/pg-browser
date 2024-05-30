use crate::common::fs::DirEntry;

pub trait PGWal {}

pub fn dir_entry() -> DirEntry {
    DirEntry::dir("pg_wal")
}
