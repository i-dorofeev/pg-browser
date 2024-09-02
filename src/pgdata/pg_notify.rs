use crate::common::fs::DirEntry;

pub trait PGNotify {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::dir("pg_notify")
}
