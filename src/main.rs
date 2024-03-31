use std::env;

use pg_browser::{
    handler::{find_handler, TermSize},
    root_handler::RootHandler,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let current_dir = env::current_dir().unwrap();
    let root_handler = Box::new(RootHandler {
        pgdata: current_dir,
    });
    let term_size = TermSize::new(&termsize::get().unwrap());
    let result = find_handler(root_handler, &args[1..])
        .map_or_else(|e| e, |handler| handler.handle(&term_size));
    println!("{result}");
}
