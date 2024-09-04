use crate::{pgdata::base::db_dir::DbDir, viewers::Viewer};

pub struct DbDirViewer<T: DbDir> {
	base_dir: T

}

impl<T> DbDirViewer<T> where T: DbDir {

	pub fn new(base_dir: T) -> Self {
		DbDirViewer { base_dir }
	}

}

impl<T: DbDir> Viewer for DbDirViewer<T> {
    fn get_next(self: Box<Self>, param: &str) -> anyhow::Result<Box<dyn Viewer>> {
        todo!()
    }

    fn handle<'a>(&self, term_size: &'a crate::viewers::TermSize, write: Box<&mut dyn std::io::prelude::Write>)
        -> anyhow::Result<()> {
        todo!()
    }
}