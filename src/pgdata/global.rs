use crate::common::fs::DirEntry;

pub trait Global {}

pub fn dir_entry() -> DirEntry<'static> {
    DirEntry::dir("global")
}
