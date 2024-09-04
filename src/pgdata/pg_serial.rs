use crate::common::fs::DirEntry;

#[allow(dead_code)]
pub trait PGSerial {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::dir("pg_serial")
}
