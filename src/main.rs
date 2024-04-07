use std::env;

use pg_browser::{
    handler::{find_handler, string_iter, TermSize},
    readers::reader_factory,
    root_handler::RootHandler,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let current_dir = env::current_dir().unwrap();
    let root_handler = Box::new(RootHandler {
        pgdata: current_dir,
    });
    let term_size = TermSize::new(&termsize::get().unwrap());
    let readers = reader_factory();
    let result = find_handler(root_handler, &args[1..]).map_or_else(string_iter, |handler| {
        handler.handle(&term_size, readers.as_ref())
    });
    for line in result {
        print!("{line}");
    }
}
