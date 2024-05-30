use crate::common::fs::DirEntry;

pub trait PGStatTmp {}

pub fn dir_entry() -> DirEntry {
    DirEntry::dir("pg_stat_tmp")
}
