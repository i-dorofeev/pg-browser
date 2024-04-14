use std::path::PathBuf;

use crate::handlers::Handler;

pub struct BaseHandler {
    pub base_path: PathBuf,
}

impl Handler for BaseHandler {
    fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String> {
        todo!()
    }

    fn handle<'a>(
        &self,
        term_size: &'a crate::handlers::TermSize,
        readers: &dyn crate::readers::ReaderFactory,
    ) -> crate::handlers::StringIter<'a> {
        todo!()
    }
}
