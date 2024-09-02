use crate::common::fs::DirEntry;

pub trait PGTblspc {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::dir("pg_tblspc")
}
