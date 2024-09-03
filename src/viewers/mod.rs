pub mod pgdata;

use std::io::Write;

pub fn find_viewer<'a>(
    viewer: Box<dyn Viewer + 'a>,
    args: &[String],
) -> anyhow::Result<Box<dyn Viewer + 'a>> {
    if args.is_empty() {
        Ok(viewer)
    } else {
        let next_viewer = viewer.get_next(&args[0]);
        find_viewer(next_viewer?, &args[1..])
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

pub trait Viewer {
    fn get_next(self: Box<Self>, param: &str) -> anyhow::Result<Box<dyn Viewer>>;

    fn handle<'a>(&self, term_size: &'a TermSize, write: Box<&mut dyn Write>)
        -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use super::{find_viewer, Viewer, TermSize};

    const TERM_SIZE: TermSize = TermSize { rows: 20, cols: 80 };

    #[test]
    fn traverses_args_and_returns_handler() {
        // given
        let root_handler = Box::new(MockHandler {
            collected_args: [].to_vec(),
        });
        let args: Vec<String> = ["a", "b", "c"].iter().map(|&i| i.to_string()).collect();

        // when
        let found_handler = find_viewer(root_handler, &args).expect("handler is found");

        let mut buf = Vec::new();
        found_handler
            .handle(&TERM_SIZE, Box::new(&mut buf))
            .unwrap();
        let output = String::from_utf8(buf).unwrap();

        // then
        assert_eq!(
            "a b c / TermSize { rows: 20, cols: 80 }", output,
            "mock handler should collect all the arguments"
        );
    }

    #[test]
    fn returns_error_when_handler_does_not_support_parameter() {
        // given
        let root_handler = Box::new(ErrHandler {});
        let args: Vec<String> = vec!["aaa".to_string()];

        // when
        let result = find_viewer(root_handler, &args);

        // then
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "aaa is not supported".to_string()
        );
    }

    struct MockHandler {
        collected_args: Vec<String>,
    }

    impl Viewer for MockHandler {
        fn get_next(self: Box<Self>, param: &str) -> anyhow::Result<Box<dyn Viewer>> {
            let mut new_args = self.collected_args.to_vec();
            new_args.push(param.to_string());

            Ok(Box::new(MockHandler {
                collected_args: new_args,
            }))
        }

        fn handle<'a>(
            &self,
            term_size: &'a TermSize,
            write: Box<&mut dyn std::io::prelude::Write>,
        ) -> anyhow::Result<()> {
            write!(write, "{} / {:?}", self.collected_args.join(" "), term_size)
                .map_err(|err| anyhow!(err))
        }
    }

    struct ErrHandler {}
    impl Viewer for ErrHandler {
        fn get_next(self: Box<Self>, param: &str) -> anyhow::Result<Box<dyn Viewer>> {
            Err(anyhow!("{param} is not supported"))
        }

        fn handle<'a>(
            &self,
            _term_size: &'a TermSize,
            _write: Box<&mut dyn std::io::prelude::Write>,
        ) -> anyhow::Result<()> {
            todo!()
        }
    }
}
