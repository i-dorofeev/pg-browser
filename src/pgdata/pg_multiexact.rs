use crate::common::fs::DirEntry;

pub trait PGMultixact {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::dir("pg_multixact")
}
