use crate::common::fs::DirEntry;

pub trait PostgresqlAutoConf {}

pub fn dir_entry() -> DirEntry {
    DirEntry::file("postgresql.auto.conf")
}
