use crate::common::fs::DirEntry;

pub trait PGSubtrans {}

pub fn dir_entry() -> DirEntry {
    DirEntry::dir("pg_subtrans")
}
