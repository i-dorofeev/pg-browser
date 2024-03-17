use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    let root_handler = Box::new(RootHandler {});
    let result = find_handler(root_handler, &args[1..])
        .map_or_else(|e| e, |handler| handler.handle());
    println!("{result}");
}

fn find_handler(
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

trait Handler {
    fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String>;
    fn handle(&self) -> String;
}

struct RootHandler {}

impl Handler for RootHandler {
    fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String> {
        match param {
            "a" => Ok(Box::new(AHandler {})),
            "b" => Ok(Box::new(BHandler {})),
            val => Ok(Box::new(ArbHandler { val: String::from(val) })),
        }
    }

    fn handle(&self) -> String {
        "provide a handler name".to_string()
    }
}

struct AHandler {}
impl Handler for AHandler {
    fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String> {
        Err(format!("AHandler: Unknown param {param}"))
    }

    fn handle(&self) -> String {
        "Handled by AHandler".to_string()
    }
}

struct BHandler {}
impl Handler for BHandler {
    fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String> {
        Err(format!("BHandler: Unknown param {param}"))
    }

    fn handle(&self) -> String {
        "Handled by BHandler".to_string()
    }
}
struct ArbHandler {
    val: String,
}
impl Handler for ArbHandler {
    fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String> {
        let this_val = &self.val;
        Ok(Box::from(ArbHandler { val: format!("{this_val}/{param}") }))
    }

    fn handle(&self) -> String {
        self.val.clone()
    }
}
