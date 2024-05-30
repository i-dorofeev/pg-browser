use crate::common::fs::DirEntry;

pub trait PGCommitTS {}

pub fn dir_entry() -> DirEntry {
    DirEntry::dir("pg_commit_ts")
}
