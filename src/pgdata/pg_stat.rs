use crate::common::fs::DirEntry;

pub trait PGStat {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::dir("pg_stat")
}
