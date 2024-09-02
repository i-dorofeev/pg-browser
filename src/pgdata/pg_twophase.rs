use crate::common::fs::DirEntry;

pub trait PGTwophase {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::dir("pg_twophase")
}
