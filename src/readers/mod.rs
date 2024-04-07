use std::path::PathBuf;

use self::root_dir_reader::{default_root_dir_reader, RootDirReader};

pub mod root_dir_reader;

pub trait ReaderFactory {
    fn root_dir_reader<'a>(&self, pgdata: &'a PathBuf) -> Box<dyn RootDirReader<'a> + 'a> {
        Box::new(default_root_dir_reader(pgdata))
    }
}

struct DefaultReaderFactory;
impl ReaderFactory for DefaultReaderFactory {}

pub fn reader_factory() -> Box<dyn ReaderFactory> {
    Box::new(DefaultReaderFactory)
}
