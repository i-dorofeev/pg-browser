use std::path::Path;

use self::{
    root_dir_reader::{root_dir_reader, RootDirReader},
    root_dir_readers::base_reader::{base_dir_reader, BaseDirReader},
};

pub mod database_file_layout;
pub mod root_dir_reader;
pub mod root_dir_readers;

pub trait ReaderFactory {
    fn root_dir_reader<'a>(&self, pgdata: &'a Path) -> Box<dyn RootDirReader + 'a> {
        Box::new(root_dir_reader(pgdata))
    }

    fn base_dir_reader<'a>(&self, base_dir_path: &'a Path) -> Box<dyn BaseDirReader + 'a> {
        Box::new(base_dir_reader(base_dir_path))
    }
}

struct DefaultReaderFactory;
impl ReaderFactory for DefaultReaderFactory {}

pub fn reader_factory() -> Box<dyn ReaderFactory> {
    Box::new(DefaultReaderFactory)
}
