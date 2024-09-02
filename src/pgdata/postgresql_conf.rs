use crate::common::fs::DirEntry;

pub trait PostgresqlConf {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::file("postgresql.conf")
}
