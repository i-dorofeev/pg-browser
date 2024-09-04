use crate::common::fs::DirEntry;

#[allow(dead_code)]
pub trait PGIdentConf {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::file("pg_ident.conf")
}
