use crate::common::fs::DirEntry;

#[allow(dead_code)]
pub trait PgHbaConf {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::file("pg_hba.conf")
}
