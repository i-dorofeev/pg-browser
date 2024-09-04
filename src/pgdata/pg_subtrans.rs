use crate::common::fs::DirEntry;

#[allow(dead_code)]
pub trait PGSubtrans {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::dir("pg_subtrans")
}
