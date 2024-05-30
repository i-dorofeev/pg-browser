use crate::common::fs::DirEntry;

pub trait PGTwophase {}

pub fn dir_entry() -> DirEntry {
    DirEntry::dir("pg_twophase")
}
