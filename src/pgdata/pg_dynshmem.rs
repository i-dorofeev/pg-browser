use crate::common::fs::DirEntry;

pub trait PGDynshmem {}

pub fn dir_entry() -> DirEntry {
    DirEntry::dir("pg_dynshmem")
}
