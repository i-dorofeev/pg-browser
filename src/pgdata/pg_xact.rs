use crate::common::fs::DirEntry;

pub trait PGXact {}

pub fn dir_entry() -> DirEntry {
    DirEntry::dir("pg_xact")
}
