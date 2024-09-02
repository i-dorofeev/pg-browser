use crate::common::fs::DirEntry;

pub trait PGSubtrans {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::dir("pg_subtrans")
}
