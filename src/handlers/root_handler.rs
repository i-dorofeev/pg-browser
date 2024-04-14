use std::path::PathBuf;

use colored::{Color, Colorize};

use crate::readers::{
    root_dir_reader::{PgDataItem, PgDataItemState, PgDataItemType},
    ReaderFactory,
};

use super::{string_iter, Handler, StringIter, TermSize};

pub struct RootHandler {
    pub pgdata: PathBuf,
}

impl Handler for RootHandler {
    fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String> {
        match param {
            "a" => Ok(Box::new(AHandler {})),
            "b" => Ok(Box::new(BHandler {})),
            val => Ok(Box::new(ArbHandler {
                val: String::from(val),
            })),
        }
    }

    fn handle<'a>(&self, term_size: &'a TermSize, readers: &dyn ReaderFactory) -> StringIter<'a> {
        let root_dir_reader = readers.root_dir_reader(&self.pgdata);
        let pgdata_items = root_dir_reader.known_pgdata_items();
        let name_col_width = pgdata_items
            .iter()
            .map(|item| item.name.len())
            .max()
            .unwrap_or(0);

        Box::new(
            pgdata_items
                .into_iter()
                .map(move |item| Self::format_pgdata_item(item, name_col_width, term_size.cols))
                .map(|item_str| format!("\n{item_str}")),
        )
    }
}

const GRAY: Color = Color::TrueColor {
    r: 127,
    g: 127,
    b: 127,
};

impl RootHandler {
    fn format_pgdata_item(
        item: PgDataItem,
        name_col_width: usize,
        terminal_width: usize,
    ) -> String {
        let item_type = match &item.item_type {
            PgDataItemType::Dir => "D",
            PgDataItemType::File => "F",
        };
        let item_type_colored = match &item.state {
            PgDataItemState::Present => item_type.green(),
            PgDataItemState::Missing => item_type.yellow(),
            PgDataItemState::Error(_) => item_type.red(),
        };

        let padded_name = format!("{name: <width$}", name = item.name, width = name_col_width);
        let padded_name_colored = match item.state {
            PgDataItemState::Present => padded_name.blue(),
            PgDataItemState::Missing => padded_name.color(GRAY),
            PgDataItemState::Error(_) => padded_name.red(),
        };

        let description_padding = item_type.len() + 1 + padded_name.len() + 1;
        let description_col_width = terminal_width - description_padding;

        let mut output = String::new();
        output.push_str(&format!(
            "{} {} {}",
            item_type_colored,
            padded_name_colored,
            Self::split(item.description, 0, description_col_width)
        ));

        (1..)
            .map(|n| Self::split(item.description, n, description_col_width))
            .take_while(|slice| !slice.is_empty())
            .map(|slice| {
                format!(
                    "\n{padding: >padding_width$}{slice}",
                    padding = "",
                    padding_width = description_padding
                )
            })
            .for_each(|slice| {
                output.push_str(&slice);
            });

        output
    }

    fn split(str: &str, n: usize, size: usize) -> String {
        str.chars().skip(size * n).take(size).collect()
    }
}

struct AHandler {}
impl Handler for AHandler {
    fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String> {
        Err(format!("AHandler: Unknown param {param}"))
    }

    fn handle<'a>(&self, _term_size: &'a TermSize, _readers: &dyn ReaderFactory) -> StringIter<'a> {
        string_iter("Handled by AHandler".to_string())
    }
}

struct BHandler {}
impl Handler for BHandler {
    fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String> {
        Err(format!("BHandler: Unknown param {param}"))
    }

    fn handle<'a>(&self, _term_size: &'a TermSize, _readers: &dyn ReaderFactory) -> StringIter<'a> {
        string_iter("Handled by BHandler".to_string())
    }
}

struct ArbHandler {
    val: String,
}
impl Handler for ArbHandler {
    fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String> {
        let this_val = &self.val;
        Ok(Box::from(ArbHandler {
            val: format!("{this_val}/{param}"),
        }))
    }

    fn handle<'a>(&self, _term_size: &'a TermSize, _readers: &dyn ReaderFactory) -> StringIter<'a> {
        string_iter(self.val.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::{io, path::Path};

    use colored::{Color, Colorize};

    use crate::{
        handlers::{root_handler::tests::colors::*, Handler, TermSize},
        readers::{
            root_dir_reader::{PgDataItem, PgDataItemState, PgDataItemType, RootDirReader},
            ReaderFactory,
        },
    };

    use super::RootHandler;

    struct RootDirReaderStub;
    impl RootDirReader for RootDirReaderStub {
        fn known_pgdata_items(&self) -> Vec<PgDataItem> {
            vec![
                PgDataItem {
                    name: "present_file.aa",
                    description: "word1word2word3word4",
                    item_type: PgDataItemType::File,
                    state: PgDataItemState::Present,
                },
                PgDataItem {
                    name: "missing_file.bbbb",
                    description: "word5word6word7word8",
                    item_type: PgDataItemType::File,
                    state: PgDataItemState::Missing,
                },
                PgDataItem {
                    name: "error_file.ccc",
                    description: "word9",
                    item_type: PgDataItemType::File,
                    state: PgDataItemState::Error(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "permission error",
                    )),
                },
                PgDataItem {
                    name: "present_dir.aa",
                    description: "word1word2word3word4",
                    item_type: PgDataItemType::Dir,
                    state: PgDataItemState::Present,
                },
                PgDataItem {
                    name: "missing_dir.bbbb",
                    description: "word5word6word7word8",
                    item_type: PgDataItemType::Dir,
                    state: PgDataItemState::Missing,
                },
                PgDataItem {
                    name: "error_dir.ccc",
                    description: "word9",
                    item_type: PgDataItemType::Dir,
                    state: PgDataItemState::Error(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "permission error",
                    )),
                },
            ]
        }
    }

    struct ReaderFactoryStub;
    impl ReaderFactory for ReaderFactoryStub {
        fn root_dir_reader<'a>(&self, _pgdata: &'a Path) -> Box<dyn RootDirReader + 'a> {
            Box::new(RootDirReaderStub)
        }
    }

    mod colors {
        use colored::Color;

        pub const BLUE: Option<Color> = Some(Color::Blue);
        pub const GRAY: Option<Color> = Some(super::super::GRAY);
        pub const GREEN: Option<Color> = Some(Color::Green);
        pub const RED: Option<Color> = Some(Color::Red);
        pub const YELLOW: Option<Color> = Some(Color::Yellow);
        pub const NONE: Option<Color> = None;
    }

    #[test]
    fn root_hander_renders_root_dir_contents() {
        // given
        let root_handler = RootHandler {
            pgdata: "/pgdata".into(),
        };

        let term_size = TermSize {
            rows: 100,
            cols: 30,
        };

        let readers = ReaderFactoryStub;

        // when
        let result = root_handler.handle(&term_size, &readers);

        // then
        fn line(str: &str, colors: &[Option<Color>]) -> String {
            let p = str.split('|');
            let line = p
                .zip(colors)
                .map(|(s, color)| {
                    color.map_or_else(|| s.to_string(), |c| format!("{}", s.color(c)))
                })
                .collect::<Vec<String>>()
                .concat();
            format!("\n{}", line)
        }

        #[rustfmt::skip]
        assert_eq!(
            result.collect::<Vec<String>>().concat(),
            [
                line("F| |present_file.aa  | |word1word2", &[ GREEN, NONE, BLUE, NONE, NONE]),
                line(" | |                 | |word3word4", &[  NONE, NONE, NONE, NONE, NONE]),
                line("F| |missing_file.bbbb| |word5word6", &[YELLOW, NONE, GRAY, NONE, NONE]),
                line(" | |                 | |word7word8", &[  NONE, NONE, NONE, NONE, NONE]),
                line("F| |error_file.ccc   | |word9",      &[   RED, NONE,  RED, NONE, NONE]),
                line("D| |present_dir.aa   | |word1word2", &[ GREEN, NONE, BLUE, NONE, NONE]),
                line(" | |                 | |word3word4", &[  NONE, NONE, NONE, NONE, NONE]),
                line("D| |missing_dir.bbbb | |word5word6", &[YELLOW, NONE, GRAY, NONE, NONE]),
                line(" | |                 | |word7word8", &[  NONE, NONE, NONE, NONE, NONE]),
                line("D| |error_dir.ccc    | |word9",      &[   RED, NONE,  RED, NONE, NONE]),

            ]
            .concat()
        );
    }
}
