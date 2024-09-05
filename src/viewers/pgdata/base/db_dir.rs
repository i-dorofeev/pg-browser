use std::io::prelude::Write;

use crate::{
    pgdata::base::db_dir::DbDir,
    viewers::{TermSize, Viewer},
};

#[allow(dead_code)]
pub struct DbDirViewer<T: DbDir> {
    db_dir: T,
}

impl<T> DbDirViewer<T>
where
    T: DbDir,
{
    pub fn new(db_dir: T) -> Self {
        DbDirViewer { db_dir }
    }
}

impl<T: DbDir> Viewer for DbDirViewer<T> {
    fn get_next(self: Box<Self>, param: &str) -> anyhow::Result<Box<dyn Viewer>> {
        anyhow::bail!("${param} not supported")
    }

    fn handle(&self, _term_size: &TermSize, _write: Box<&mut dyn Write>) -> anyhow::Result<()> {
        anyhow::bail!("DbDirViewer is not implemented")
    }
}
