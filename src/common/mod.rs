use std::ffi::OsStr;

use anyhow::anyhow;

pub mod fs;
pub mod result_option;

#[cfg(test)]
pub mod test_utils;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PgOid(pub u32);

impl PgOid {
    // TODO: tests for PgOid::try_parse
    pub fn try_parse<T>(s: T) -> Option<Self>
    where
        T: AsRef<OsStr>,
    {
        match s.as_ref().to_string_lossy().parse::<u32>() {
            Ok(oid) => Some(PgOid(oid)),
            Err(_) => None,
        }
    }
}

impl From<u32> for PgOid {
    fn from(value: u32) -> Self {
        PgOid(value)
    }
}

pub fn stringify(err: anyhow::Error) -> String {
    format!("{:?}", err)
}

#[derive(Debug)]
pub struct Error(pub anyhow::Error);
impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        let Error(err_this) = self;
        let Error(err_other) = other;
        format!("{:?}", err_this) == format!("{:?}", err_other)
    }
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Error(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error(anyhow!(error))
    }
}
