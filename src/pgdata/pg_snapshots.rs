use crate::common::fs::DirEntry;

pub trait PGSnapshots {}
pub fn dir_entry() -> DirEntry {
    DirEntry::dir("pg_snapshots")
}
