use crate::common::fs::DirEntry;

#[allow(dead_code)]
pub trait PGNotify {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::dir("pg_notify")
}
