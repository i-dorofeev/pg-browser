use crate::common::fs::DirEntry;

pub trait PgHbaConf {}

pub fn dir_entry() -> DirEntry {
    DirEntry::file("pg_hba.conf")
}
