pub fn find_handler(
    root_handler: Box<dyn Handler>,
    args: &[String],
) -> Result<Box<dyn Handler>, String> {
    if args.is_empty() {
        Ok(root_handler)
    } else {
        let next_handler = root_handler.get_next(&args[0]);
        find_handler(next_handler?, &args[1..])
    }
}

pub trait Handler {
    fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String>;
    fn handle(&self) -> String;
}