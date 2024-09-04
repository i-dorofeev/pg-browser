use crate::common::fs::DirEntry;

#[allow(dead_code)]
pub trait PGStatTmp {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::dir("pg_stat_tmp")
}
