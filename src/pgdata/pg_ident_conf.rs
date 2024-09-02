use crate::common::fs::DirEntry;

pub trait PGIdentConf {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::file("pg_ident.conf")
}
