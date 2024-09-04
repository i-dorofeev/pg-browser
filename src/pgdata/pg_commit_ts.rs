use crate::common::fs::DirEntry;

#[allow(dead_code)]
pub trait PGCommitTS {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::dir("pg_commit_ts")
}
