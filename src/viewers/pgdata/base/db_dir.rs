use std::{cmp::Ordering, collections::BinaryHeap, fmt::Display, hash::{DefaultHasher, Hash, Hasher}, io::prelude::Write};

use anyhow::anyhow;

use crate::{
    common::{self, fs::DirEntry, into_ordered_iterator::IntoOrderedIterator, PgOid},
    pgdata::base::db_dir::{DbDir, DbDirItem, ForkSegmentFile, ForkType},
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

    fn handle(&self, _term_size: &TermSize, mut write: Box<&mut dyn Write>) -> anyhow::Result<()> {
        //let db_dir_items = self.db_dir.items()?;
        let db_dir_items = vec![
            DbDirItem::fork_segment_file("12345_fsm").unwrap().unwrap(),
            DbDirItem::fork_segment_file("1234_vm.1").unwrap().unwrap(),
            DbDirItem::fork_segment_file("12345.2").unwrap().unwrap(),
            DbDirItem::fork_segment_file("12345_vm").unwrap().unwrap(),
            DbDirItem::PgVersionFile,
            DbDirItem::fork_segment_file("1234_fsm.2").unwrap().unwrap(),
            DbDirItem::fork_segment_file("12345").unwrap().unwrap(),
            DbDirItem::UnknownEntry(DirEntry::file("some_file")),
            DbDirItem::fork_segment_file("1234_vm.2").unwrap().unwrap(),
            DbDirItem::fork_segment_file("1234").unwrap().unwrap(),
            DbDirItem::Error(anyhow!("Some error").into()),
            DbDirItem::fork_segment_file("1234_vm").unwrap().unwrap(),
            DbDirItem::fork_segment_file("12345.1").unwrap().unwrap(),
            DbDirItem::fork_segment_file("12345_vm.1").unwrap().unwrap(),
            DbDirItem::fork_segment_file("1234.2").unwrap().unwrap(),
            DbDirItem::fork_segment_file("1234.1").unwrap().unwrap(),
            DbDirItem::fork_segment_file("12345_vm.2").unwrap().unwrap(),
            DbDirItem::FileNodeMapFile,
            DbDirItem::UnknownEntry(DirEntry::dir("some_dir")),
            DbDirItem::fork_segment_file("1234_fsm").unwrap().unwrap(),
            DbDirItem::fork_segment_file("12345_fsm.2")
                .unwrap()
                .unwrap(),
            DbDirItem::fork_segment_file("12345_fsm.1")
                .unwrap()
                .unwrap(),
            DbDirItem::fork_segment_file("1234_fsm.1").unwrap().unwrap(),
        ];

        let ordered_db_dir_items = db_dir_items
            .into_iter()
            .map(Into::<OrderedDbDirItem>::into)
            .collect::<BinaryHeap<_>>();

        let mut _alt = false;
        let mut term_style = EMPTY;
        let mut alt_term_style = term_style.with(Style::BackgroundColor(TermColor { r: 30, g: 30, b: 30 }));
        let mut alt = Alternate::new([term_style, alt_term_style], 0u64);

        let mut block_id: u64 = 0;

        ordered_db_dir_items.into_ordered_iter().try_for_each(|item| {
            let style = alt.option(item.block_id());
            // _alt = if current_block_id != block_id {
            //     block_id = current_block_id;
            //     !_alt
            // } else {
            //     _alt
            // };
            // let current_term_style = if _alt {
            //     &term_style
            // } else {
            //     &alt_term_style
            // };
            write!(write, "{}", item.display(style))?;
            write!(write, "{}", EMPTY)?;
            writeln!(write)
        })?;
        
        Ok(())
    }
}

struct Alternate<T, U, const N: usize> where U: PartialEq {
    options: [T; N],
    index: usize,
    state: U
}

impl<T, U, const N: usize> Alternate<T, U, N> where U: PartialEq {

    fn new(options: [T; N], state: U) -> Self {
        Alternate { options, index: 0, state }
    }

    fn option(&mut self, state: U) -> &T {
        if state != self.state {
            self.index = (self.index + 1) % N;
        }

        &self.options[self.index]
    }

    fn by_index(&self, index: usize) -> &T {
        &self.options[index]
    }

}

struct TermStyle<'a> {
    style: Style,
    prev: Option<&'a TermStyle<'a>>
}

enum Style {
    None,
    BackgroundColor(TermColor),
    ForegroundColor(TermColor),
}

impl Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Style::BackgroundColor(color) => write!(f, "48;{color}"),
            Style::ForegroundColor(color) => write!(f, "38;{color}"),
            Style::None => write!(f, "0")
        }
    }
}

const EMPTY: TermStyle<'static> = TermStyle { style: Style::None, prev: None };

impl TermStyle<'_> {
    pub fn new(style: Style) -> Self {
        TermStyle {
            style,
            prev: None
        }
    }

    pub fn empty() -> Self {
        Self::new(Style::None)
    }

    pub fn with(&self, style: Style) -> TermStyle<'_> {
        TermStyle { style, prev: Some(self) }
    }
}

impl TermStyle<'_> {
    fn write_esc_seq_start(f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\x1B[")
    }

    fn write_esc_seq_end(f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "m")
    }

    // fn write_color<T>(
    //     color_code: &'static str,
    //     color_spec: Option<T>,
    //     f: &mut std::fmt::Formatter<'_>,
    // ) -> std::fmt::Result
    // where
    //     T: Display,
    // {
    //     match color_spec {
    //         Some(spec) => {
    //             write!(f, "{color_code};{spec}")
    //         }
    //         None => Ok(()),
    //     }
    // }

    // fn write_separator(comma: bool, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //     if comma {
    //         write!(f, ";")?;
    //     }
    //     Ok(())
    // }

    fn write_reset(f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0")
    }

    fn write_style_codes(style: &TermStyle<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(prev_style) = style.prev {
            Self::write_style_codes(prev_style, f)?;
        }

        write!(f, ";{}", style.style)
    }
}

impl Display for TermStyle<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        TermStyle::write_esc_seq_start(f)?;
        TermStyle::write_reset(f)?;
        TermStyle::write_style_codes(self, f)?;
        TermStyle::write_esc_seq_end(f)
    }
}

#[derive(PartialEq, Clone, Copy)]
struct TermColor {
    r: u8,
    g: u8,
    b: u8,
}

impl Display for TermColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "2;{};{};{}", self.r, self.g, self.b)
    }
}

trait StyledDisplay: Sized {

    fn fmt(&self, style: &TermStyle<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;

    fn display<'a>(&'a self, style: &'a TermStyle<'_>) -> impl Display + 'a {
        StyledDisplayImpl { style, styled_display: self }

    }
}

struct StyledDisplayImpl<'a, 'b, T>
    where T: StyledDisplay + Sized
{
    style: &'a TermStyle<'a>,
    styled_display: &'b T,
}

impl<T> Display for StyledDisplayImpl<'_, '_, T> where T: StyledDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.style)?;
        self.styled_display.fmt(self.style, f)
    }
}

mod style {
    use colored::Color;

    pub(super) const SEGMENT_0: Color = Color::TrueColor {
        r: 100,
        g: 100,
        b: 100,
    };

    pub(super) const ALT_BACKGROUND: Color = Color::TrueColor {
        r: 30,
        g: 30,
        b: 30,
    };
}





fn render_fork_type(fork_type: &ForkType) -> &'static str {
    match fork_type {
        ForkType::Main => "",
        ForkType::FreeSpaceMap => "_fsm",
        ForkType::VisibilityMap => "_vm",
    }
}



fn fork_type_description(fork_type: &ForkType) -> &'static str {
    match fork_type {
        ForkType::Main => "main",
        ForkType::FreeSpaceMap => "free space map",
        ForkType::VisibilityMap => "visibility map",
    }
}

struct ForkSegmentFileView(ForkSegmentFile);
struct FileNodeMapFileView;
struct PgVersionFileView;
struct UnknownEntryView<'a>(DirEntry<'a>);
struct ErrorView(common::Error);

enum OrderedDbDirItem<'a> {
    ForkSegmentFile(ForkSegmentFileView),
    FileNodeMapFile(FileNodeMapFileView),
    PgVersionFile(PgVersionFileView),
    UnknownEntry(UnknownEntryView<'a>),
    Error(ErrorView)
}

impl<'a> From<DbDirItem<'a>> for OrderedDbDirItem<'a> {
    fn from(value: DbDirItem<'a>) -> Self {
        match value {
            DbDirItem::ForkSegmentFile(file) => OrderedDbDirItem::ForkSegmentFile(ForkSegmentFileView(file)),
            DbDirItem::FileNodeMapFile => OrderedDbDirItem::FileNodeMapFile(FileNodeMapFileView),
            DbDirItem::PgVersionFile => OrderedDbDirItem::PgVersionFile(PgVersionFileView),
            DbDirItem::UnknownEntry(entry) => OrderedDbDirItem::UnknownEntry(UnknownEntryView(entry)),
            DbDirItem::Error(error) => OrderedDbDirItem::Error(ErrorView(error)),
        }
    }
}

impl StyledDisplay for OrderedDbDirItem<'_> {
    fn fmt(&self, style: &TermStyle<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderedDbDirItem::ForkSegmentFile(view) => view.fmt(style, f),
            OrderedDbDirItem::FileNodeMapFile(view) => view.fmt(style, f),
            OrderedDbDirItem::PgVersionFile(view) => view.fmt(style, f),
            OrderedDbDirItem::UnknownEntry(view) => view.fmt(style, f),
            OrderedDbDirItem::Error(view) => view.fmt(style, f),
        }
    }
}

impl StyledDisplay for ForkSegmentFileView {
    fn fmt(&self, style: &TermStyle<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ForkSegmentFileView(file) = self;
        write!(f, "{}", file.oid())?;
        write!(f, "{}", render_fork_type(&file.fork_type()))?;

        if file.segment_id() == 0 {
            write!(f, "{}", style.with(Style::ForegroundColor(TermColor { r: 100, g: 100, b: 100 })))?;
        }

        write!(f, ".{}", file.segment_id())?;
        write!(f, "{}", style)?;

        write!(f, "{:<20}", " ")?;
        write!(f, "{} segment #{}", fork_type_description(&file.fork_type()),
                    file.segment_id())
    }
}

impl StyledDisplay for FileNodeMapFileView {
    fn fmt(&self, _style: &TermStyle<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<42}", "pg_filenode.map")
    }
}

impl StyledDisplay for PgVersionFileView {
    fn fmt(&self, _style: &TermStyle<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<42}", "PG_VERSION") 
    }
}

impl StyledDisplay for UnknownEntryView<'_> {
    fn fmt(&self, _style: &TermStyle<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:?}", self.0.entry_type, self.0.name)
    }
}

impl StyledDisplay for ErrorView {
    fn fmt(&self, _style: &TermStyle<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl PartialOrd for OrderedDbDirItem<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedDbDirItem<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.oid().cmp(&other.oid()).reverse().then(
            fork_type_order(&self.fork_type())
                .cmp(&fork_type_order(&other.fork_type()))
                .then(self.segment_id().cmp(&other.segment_id()))
                .reverse(),
        )
    }
}

impl PartialEq for OrderedDbDirItem<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.oid().eq(&other.oid())
    }
}

impl Eq for OrderedDbDirItem<'_> {}

impl OrderedDbDirItem<'_> {
    fn oid(&self) -> Option<PgOid> {
        match self {
            OrderedDbDirItem::ForkSegmentFile(ForkSegmentFileView(file)) => Some(file.oid()),
            _ => None,
        }
    }

    fn fork_type(&self) -> Option<ForkType> {
        match self {
            OrderedDbDirItem::ForkSegmentFile(ForkSegmentFileView(file)) => Some(file.fork_type()),
            _ => None,
        }
    }

    fn segment_id(&self) -> Option<u16> {
        match self {
            OrderedDbDirItem::ForkSegmentFile(ForkSegmentFileView(file)) => Some(file.segment_id()),
            _ => None,
        }
    }

    fn block_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        match self {
            OrderedDbDirItem::ForkSegmentFile(ForkSegmentFileView(file)) => file.oid().hash(&mut hasher),
            OrderedDbDirItem::FileNodeMapFile(_) => "fileNodeMapFile".hash(&mut hasher),
            OrderedDbDirItem::PgVersionFile(_) => "pg_version".hash(&mut hasher),
            OrderedDbDirItem::UnknownEntry(UnknownEntryView(dir_entry)) => dir_entry.name.hash(&mut hasher),
            OrderedDbDirItem::Error(ErrorView(error)) => format!("{:?}", error).hash(&mut hasher),
        };
        hasher.finish()
    }
}

fn fork_type_order(fork_type: &Option<ForkType>) -> u8 {
    match fork_type {
        Some(ForkType::Main) => 1,
        Some(ForkType::FreeSpaceMap) => 2,
        Some(ForkType::VisibilityMap) => 3,
        None => 4,
    }
}
