use crate::common::fs::DirEntry;

#[allow(dead_code)]
pub trait PGStat {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::dir("pg_stat")
}
