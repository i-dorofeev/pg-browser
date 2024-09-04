use crate::common::fs::DirEntry;

#[allow(dead_code)]
pub trait Global {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::dir("global")
}
