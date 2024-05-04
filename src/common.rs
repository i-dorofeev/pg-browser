use std::fs::FileType as FsFileType;

use anyhow::anyhow;

#[derive(Debug, PartialEq)]
pub struct PgOid(pub u32);

#[derive(Debug, PartialEq)]
pub enum FileType {
    Dir,
    File,
}

impl FileType {
    pub fn from(file_type: FsFileType) -> anyhow::Result<FileType> {
        match file_type {
            _ if file_type.is_dir() => Ok(FileType::Dir),
            _ if file_type.is_file() => Ok(FileType::File),
            _ => Err(anyhow!("Unknown FileType {:?}", file_type)),
        }
    }
}

pub fn render_file_type(file_type: &FileType) -> String {
    match file_type {
        FileType::Dir => "D",
        FileType::File => "F",
    }
    .to_string()
}
