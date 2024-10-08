use std::{env, io::stdout};

use anyhow::{Context, Ok};
use pg_browser::{
    pgdata,
    viewers::{find_viewer, pgdata::RootViewer, TermSize},
};

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();
    let current_dir = env::current_dir().context("Can't get current directory")?;
    let pgdata = pgdata::pgdata(current_dir.into());
    let root_viewer = Box::new(RootViewer { pgdata });
    let term_size = termsize::get()
        .context("termsize not available")
        .map(|size| TermSize::new(&size))?;
    let mut stdout = stdout();
    let viewer = find_viewer(root_viewer, &args[1..])?;
    viewer.handle(&term_size, Box::new(&mut stdout))?;
    Ok(())
}
