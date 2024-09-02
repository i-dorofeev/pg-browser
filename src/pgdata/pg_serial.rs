use crate::common::fs::DirEntry;

pub trait PGSerial {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::dir("pg_serial")
}
