use crate::common::fs::DirEntry;

#[allow(dead_code)]
pub trait CurrentLogFiles {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::file("current_logfiles")
}
