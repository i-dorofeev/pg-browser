#![cfg(test)]

use colored::Color;
use colored::Colorize;

pub fn line(str: &str, colors: &[Option<Color>]) -> String {
    let p = str.split('|');
    let line = p
        .zip(colors)
        .map(|(s, color)| color.map_or_else(|| s.to_string(), |c| format!("{}", s.color(c))))
        .collect::<Vec<String>>()
        .concat();
    format!("{}", line)
}

pub mod colors {
    use colored::Color;

    pub const BLUE: Option<Color> = Some(Color::Blue);
    pub const BRIGHT_BLUE: Option<Color> = Some(Color::BrightBlue);
    pub const GRAY: Option<Color> = Some(crate::GRAY);
    pub const GREEN: Option<Color> = Some(Color::Green);
    pub const RED: Option<Color> = Some(Color::Red);
    pub const YELLOW: Option<Color> = Some(Color::Yellow);
    pub const NONE: Option<Color> = None;
}
