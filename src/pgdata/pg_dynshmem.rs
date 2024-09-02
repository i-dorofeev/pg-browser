use crate::common::fs::DirEntry;

pub trait PGDynshmem {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::dir("pg_dynshmem")
}
