use std::path::PathBuf;

use colored::{Color, Colorize};

use crate::{
    handler::{string_iter, Handler, StringIter, TermSize},
    readers::{
        root_dir_reader::{PgDataItem, PgDataItemState, PgDataItemType},
        ReaderFactory,
    },
};

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
            PgDataItemState::Missing => padded_name.color(Color::TrueColor {
                r: 127,
                g: 127,
                b: 127,
            }),
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
            .for_each(|slice| {
                output.push_str(&format!(
                    "\n{padding: >padding_width$}{slice}",
                    padding = "",
                    padding_width = description_padding
                ));
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
