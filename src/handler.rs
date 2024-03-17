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

#[cfg(test)]
mod tests {
    use super::{find_handler, Handler};

    #[test]
    fn traverses_args_and_returns_handler() {
        // given
        let root_handler = Box::new(MockHandler { collected_args: [].to_vec() });
        let args: Vec<String> = ["a", "b", "c"].iter().map(|&i| i.to_string()).collect();

        // when
        let found_handler = find_handler(root_handler, &args).expect("handler is found");
        let result = found_handler.handle();

        // then
        assert_eq!("a b c", &result, "mock handler should collect all the arguments");
    }

    #[test]
    fn returns_error_when_handler_does_not_support_parameter() {
        // given
        let root_handler = Box::new(ErrHandler {});
        let args: Vec<String> = vec!["aaa".to_string()];

        // when
        let result = find_handler(root_handler , &args);

        // then
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "aaa is not supported".to_string());
    }

    struct MockHandler {
        collected_args: Vec<String>
    }

    impl Handler for MockHandler {
        fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String> {
            let mut new_args = self.collected_args.to_vec();
            new_args.push(param.to_string());

            Ok(Box::new(MockHandler { collected_args: new_args }))
        }
    
        fn handle(&self) -> String {
            self.collected_args.join(" ")
        }
    }

    struct ErrHandler { }
    impl Handler for ErrHandler {
        fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String> {
            Err(format!("{param} is not supported"))
        }
    
        fn handle(&self) -> String {
            todo!()
        }
    }

}