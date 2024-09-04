use std::io::prelude::Write;

use crate::{
    pgdata::base::db_dir::DbDir,
    viewers::{TermSize, Viewer},
};

pub struct DbDirViewer<T: DbDir> {
    base_dir: T,
}

impl<T> DbDirViewer<T>
where
    T: DbDir,
{
    pub fn new(base_dir: T) -> Self {
        DbDirViewer { base_dir }
    }
}

impl<T: DbDir> Viewer for DbDirViewer<T> {
    fn get_next(self: Box<Self>, param: &str) -> anyhow::Result<Box<dyn Viewer>> {
        anyhow::bail!("${param} not supported")
    }

    fn handle<'a>(
        &self,
        _term_size: &'a TermSize,
        _write: Box<&mut dyn Write>,
    ) -> anyhow::Result<()> {
        anyhow::bail!("DbDirViewer is not implemented")
    }
}
