use std::{path::Path, rc::Rc};

use crate::common::fs::DirEntry;

pub trait PGVersion {}

pub fn dir_entry() -> DirEntry {
    DirEntry::file("PG_VERSION")
}

pub fn pg_version(pgdata: Rc<Path>) -> impl PGVersion {
    default_impl::PGVersion::new(pgdata)
}

mod default_impl {
    use std::{path::Path, rc::Rc};

    pub struct PGVersion {
        pgdata: Rc<Path>,
    }

    impl super::PGVersion for PGVersion {}

    impl PGVersion {
        pub fn new(pgdata: Rc<Path>) -> PGVersion {
            PGVersion { pgdata }
        }
    }
}
