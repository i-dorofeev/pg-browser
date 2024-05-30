use crate::common::fs::DirEntry;

pub trait PGLogical {}

pub fn dir_entry() -> DirEntry {
    DirEntry::dir("pg_logical")
}
