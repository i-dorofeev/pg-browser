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

#[derive(Debug)]
pub struct TermSize {
    pub rows: usize,
    pub cols: usize,
}

impl TermSize {
    pub fn new(size: &termsize::Size) -> TermSize {
        TermSize {
            rows: size.rows as usize,
            cols: size.cols as usize,
        }
    }
}

pub type StringIter<'a> = Box<dyn Iterator<Item = String> + 'a>;

pub fn string_iter<'a>(str: String) -> StringIter<'a> {
    Box::new(vec![str].into_iter())
}

pub trait Handler {
    fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String>;
    fn handle<'a>(&self, term_size: &'a TermSize) -> StringIter<'a>;
}

#[cfg(test)]
mod tests {
    use super::{find_handler, Handler, StringIter, TermSize};

    const TERM_SIZE: TermSize = TermSize { rows: 20, cols: 80 };

    #[test]
    fn traverses_args_and_returns_handler() {
        // given
        let root_handler = Box::new(MockHandler {
            collected_args: [].to_vec(),
        });
        let args: Vec<String> = ["a", "b", "c"].iter().map(|&i| i.to_string()).collect();

        // when
        let found_handler = find_handler(root_handler, &args).expect("handler is found");
        let result = found_handler.handle(&TERM_SIZE);

        // then
        assert_eq!(
            &vec!["a b c / TermSize { rows: 20, cols: 80 }".to_string()],
            &result.collect::<Vec<String>>(),
            "mock handler should collect all the arguments"
        );
    }

    #[test]
    fn returns_error_when_handler_does_not_support_parameter() {
        // given
        let root_handler = Box::new(ErrHandler {});
        let args: Vec<String> = vec!["aaa".to_string()];

        // when
        let result = find_handler(root_handler, &args);

        // then
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "aaa is not supported".to_string());
    }

    struct MockHandler {
        collected_args: Vec<String>,
    }

    impl Handler for MockHandler {
        fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String> {
            let mut new_args = self.collected_args.to_vec();
            new_args.push(param.to_string());

            Ok(Box::new(MockHandler {
                collected_args: new_args,
            }))
        }

        fn handle<'a>(&self, term_size: &'a TermSize) -> StringIter<'a> {
            Box::new(
                vec![format!(
                    "{} / {:?}",
                    self.collected_args.join(" "),
                    term_size
                )]
                .into_iter(),
            )
        }
    }

    struct ErrHandler {}
    impl Handler for ErrHandler {
        fn get_next(&self, param: &str) -> Result<Box<dyn Handler>, String> {
            Err(format!("{param} is not supported"))
        }

        fn handle<'a>(&self, _term_size: &'a TermSize) -> StringIter<'a> {
            todo!()
        }
    }
}
