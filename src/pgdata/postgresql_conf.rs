use crate::common::fs::DirEntry;

#[allow(dead_code)]
pub trait PostgresqlConf {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::file("postgresql.conf")
}
