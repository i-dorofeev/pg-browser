use crate::common::fs::DirEntry;

#[allow(dead_code)]
pub trait PostgresqlAutoConf {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::file("postgresql.auto.conf")
}
