use crate::{
    common::{fs::{render_file_type, DirEntry}, PgOid},
    pgdata::base::{Base, BaseDirItem}, viewers::{TermSize, Viewer},
};

use anyhow::{anyhow, Context};
use colored::Colorize;

use crate::GRAY;

use self::db_dir::DbDirViewer;

mod db_dir;

pub struct BaseViewer<T: Base> {
    pub base: T,
}

impl<T: Base> Viewer for BaseViewer<T> {
    fn get_next(self: Box<Self>, _param: &str) -> anyhow::Result<Box<dyn Viewer>> {
        let base_dir = PgOid::try_parse(_param).context("Expected database oid").and_then(|oid| self.base.db_dir(oid))?;
        Ok(Box::new(DbDirViewer::new(base_dir)))
    }

    fn handle<'a>(
        &self,
        _term_size: &'a TermSize,
        mut write: Box<&mut dyn std::io::Write>,
    ) -> anyhow::Result<()> {
        write!(
            write,
            "{}",
            self.base
                .path()
                .parent()
                .expect("pgdata path")
                .to_string_lossy()
                .color(GRAY)
        )?;
        write!(write, "{}", "/base".yellow())?;
        write!(write, "\nEach directory stores data for each database in the cluster and is named after the database's OID in {}", "pg_database".color(GRAY))?;

        let items = self.base.items()?;
        items
            .into_iter()
            .map(|item| {
                writeln!(write, "")?;
                format_base_dir_item(item, &mut write)
            })
            .collect::<anyhow::Result<()>>()?;
        writeln!(write, "").map(|_| ()).map_err(|err| anyhow!(err))
    }
}

fn format_base_dir_item(
    base_dir_item: BaseDirItem<'_>,
    target: &mut Box<&mut dyn std::io::Write>,
) -> anyhow::Result<()> {
    match base_dir_item {
        BaseDirItem::DatabaseDir(dir) => {
            // dir name is a string representation of oid
            // oid is an unsigned 32-bit integer with a range of values [0; 4,294,967,295]
            // and string representation maximum length of 10 chars
            write!(
                target,
                "D {:>10} {}",
                dir.dir_name().bright_blue(),
                dir.db_name()
            )
        }
        BaseDirItem::UnknownEntry(DirEntry { name, entry_type }) => {
            write!(
                target,
                "{} {}",
                render_file_type(&entry_type),
                name.to_string_lossy().color(GRAY)
            )
        }
        BaseDirItem::Error(err) => write!(target, "E {}", err.to_string().red()),
    }
    .map(|_| ())
    .map_err(|err| anyhow!(err))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use anyhow::anyhow;
    use pretty_assertions::assert_eq;

    use crate::common::PgOid;
    use crate::pgdata::base::db_dir::test_stubs::StubDbDir;
    use crate::pgdata::base::db_dir::DbDir;
    use crate::pgdata::base::{Base, BaseDirItem};
    use crate::viewers::{Viewer, TermSize};
    use crate::{
        test_utils::colors::{BRIGHT_BLUE, GRAY, NONE, RED, YELLOW},
        test_utils::line,
    };

    use super::BaseViewer;

    #[test]
    fn base_hander_renders_base_dir_contents() {
        // given
        let base = BaseStub {
            items: || {
                vec![
                    BaseDirItem::database_dir(2, "database_name_1"),
                    BaseDirItem::database_dir(std::u32::MAX, "database_name_2"),
                    BaseDirItem::unknown_file("some_file"),
                    BaseDirItem::unknown_dir("some_dir"),
                    BaseDirItem::Error(anyhow!("unexpected error")),
                ]
            },
        };

        let base_viewer = BaseViewer { base };

        let term_size = TermSize {
            rows: 100,
            cols: 30,
        };

        let mut buf = Vec::new();

        // when
        base_viewer.handle(&term_size, Box::new(&mut buf)).unwrap();
        let output = String::from_utf8_lossy(&buf).into_owned();

        // then
        #[rustfmt::skip]
        assert_eq!(
            output,
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

    struct BaseStub<'a, F>
    where
        F: Fn() -> Vec<BaseDirItem<'a>>,
    {
        items: F,
    }

    impl<'a, F> Base for BaseStub<'a, F>
    where
        F: Fn() -> Vec<BaseDirItem<'a>>,
    {
        fn path(&self) -> &Path {
            Path::new("/pgdata/base")
        }

        fn items(
            &self,
        ) -> anyhow::Result<
            impl IntoIterator<
                Item = crate::pgdata::base::BaseDirItem,
                IntoIter = impl Iterator<Item = crate::pgdata::base::BaseDirItem>,
            >,
        > {
            Ok((self.items)().into_iter())
        }

        fn db_dir<'b>(&self, oid: PgOid) -> anyhow::Result<impl DbDir + 'b> {
            Ok(StubDbDir {})
        }
    }
}
