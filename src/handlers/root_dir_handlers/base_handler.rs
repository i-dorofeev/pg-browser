use crate::{
    common::render_file_type,
    readers::root_dir_readers::base_reader::{BaseDirEntry, BaseDirItem},
};
use std::path::PathBuf;

use anyhow::Error;
use colored::Colorize;

use crate::{
    handlers::{Handler, StringIter},
    GRAY,
};

pub struct BaseHandler {
    pub pgdata: PathBuf,
}

impl Handler for BaseHandler {
    fn get_next(self: Box<Self>, _param: &str) -> Result<Box<dyn Handler>, String> {
        todo!()
    }

    fn handle<'a>(
        &self,
        _term_size: &'a crate::handlers::TermSize,
        readers: &dyn crate::readers::ReaderFactory,
    ) -> Result<StringIter<'a>, Error> {
        let path = self.pgdata.join("base");
        let reader = readers.base_dir_reader(path.as_path());
        let base_dir = reader.read_base_dir()?;

        let mut output = vec![];
        output.push(format!("{}", self.pgdata.to_string_lossy().color(GRAY)));
        output.push(format!("{}", "/base".yellow()));
        output.push(format!("\n{}", format!("Each directory stores data for each database in the cluster and is named after the database's OID in {}", "pg_database".color(GRAY))));
        base_dir
            .items()
            .iter()
            .map(format_base_dir_item)
            .for_each(|item| output.push(format!("\n{}", item)));
        output.push("\n".to_string());
        Ok(Box::new(output.into_iter()))
    }
}

fn format_base_dir_item(base_dir_item: &BaseDirItem) -> String {
    match base_dir_item {
        BaseDirItem::DatabaseDir(dir) => {
            // dir name is a string representation of oid
            // oid is an unsigned 32-bit integer with a range of values [0; 4,294,967,295]
            // and string representation maximum length of 10 chars
            format!("D {:>10} {}", dir.dir_name().bright_blue(), dir.db_name)
        }
        BaseDirItem::UnknownEntry(BaseDirEntry { name, entry_type }) => {
            format!(
                "{} {}",
                render_file_type(entry_type),
                name.to_string_lossy().color(GRAY)
            )
        }
        BaseDirItem::Error(err) => format!("E {}", err.to_string().red()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use anyhow::anyhow;
    use anyhow::Error;

    use crate::readers::root_dir_readers::base_reader::{BaseDir, BaseDirItem, BaseDirReader};
    use crate::{
        handlers::{Handler, TermSize},
        readers::ReaderFactory,
        test_utils::colors::{BRIGHT_BLUE, GRAY, NONE, RED, YELLOW},
        test_utils::line,
    };

    use super::BaseHandler;

    #[test]
    fn base_hander_renders_base_dir_contents() {
        // given
        let base_handler = BaseHandler {
            pgdata: "/pgdata".into(),
        };

        let term_size = TermSize {
            rows: 100,
            cols: 30,
        };

        let readers = ReaderFactoryStub;

        // when
        let result = base_handler.handle(&term_size, &readers).unwrap();

        // then
        #[rustfmt::skip]
        assert_eq!(
            result.collect::<Vec<String>>().concat(),
            [
                line("/pgdata|/base", &[GRAY, YELLOW]),
                line("Each directory stores data for each database in the cluster and is named after the database's OID in |pg_database", &[NONE, GRAY]),
                line("D |         2| database_name_1", &[NONE, BRIGHT_BLUE, NONE]),
                line("D |4294967295| database_name_2", &[NONE, BRIGHT_BLUE, NONE]),
                line("F |some_file", &[NONE, GRAY]),
                line("D |some_dir", &[NONE, GRAY]),
                line("E |unexpected error", &[NONE, RED]),
                line("", &[])
            ]
            .join("\n")
        );
    }

    struct ReaderFactoryStub;
    impl ReaderFactory for ReaderFactoryStub {
        fn base_dir_reader<'a>(&self, _base_dir_path: &'a Path) -> Box<dyn BaseDirReader + 'a> {
            Box::new(BaseDirReaderStub)
        }
    }

    struct BaseDirReaderStub;
    impl BaseDirReader for BaseDirReaderStub {
        #[rustfmt::skip]
        fn read_base_dir(&self) -> Result<BaseDir, Error> {
            Ok(BaseDir(vec![
                BaseDirItem::database_dir(2, "database_name_1"),
                BaseDirItem::database_dir(std::u32::MAX, "database_name_2"),
                BaseDirItem::unknown_file("some_file"),
                BaseDirItem::unknown_dir("some_dir"),
                BaseDirItem::Error(anyhow!("unexpected error")),
            ]))
        }
    }
}
