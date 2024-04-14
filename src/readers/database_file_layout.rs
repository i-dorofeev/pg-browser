use std::path::{Path, PathBuf};

use self::pg_data::Base;

pub struct PGData(pub PathBuf);
impl PGData {
    pub fn base(&self) -> Base {
        Base(self.0.join("base"))
    }

    pub fn path(&self) -> &Path {
        self.0.as_path()
    }
}

pub mod pg_data {
    use std::path::PathBuf;

    pub struct Base(pub PathBuf);
}
