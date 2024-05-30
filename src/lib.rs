use colored::Color;

pub mod common;
pub mod handlers;
pub mod pgdata;
pub mod readers;
pub mod test_utils;

const GRAY: Color = Color::TrueColor {
    r: 127,
    g: 127,
    b: 127,
};
