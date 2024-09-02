use std::{env, io::stdout};

use anyhow::Ok;
use pg_browser::{
    handlers::{find_handler, root_handler::RootHandler, TermSize},
    pgdata,
};

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();
    let current_dir = env::current_dir().unwrap();
    let pgdata = pgdata::pgdata(current_dir.into());
    let root_handler = Box::new(RootHandler { pgdata });
    let term_size = TermSize::new(&termsize::get().unwrap());
    let mut stdout = stdout();
    let handler = find_handler(root_handler, &args[1..])?;
    handler.handle(&term_size, Box::new(&mut stdout))?;
    Ok(())
}
