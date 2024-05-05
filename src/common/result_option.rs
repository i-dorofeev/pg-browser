use anyhow::Result;

pub trait ResultOption<T> {
    fn or_if_empty<O>(self, op: O) -> Result<Option<T>>
    where
        O: FnOnce() -> Result<Option<T>>;

    fn otherwise<O>(self, op: O) -> Result<T>
    where
        O: FnOnce() -> Result<T>;
}

impl<T> ResultOption<T> for Result<Option<T>> {
    fn or_if_empty<O: FnOnce() -> Result<Option<T>>>(self, op: O) -> Result<Option<T>> {
        match self {
            Ok(None) => op(),
            _ => self,
        }
    }

    fn otherwise<O: FnOnce() -> Result<T>>(self, op: O) -> Result<T> {
        match self {
            Ok(None) => op(),
            Ok(Some(v)) => Ok(v),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::stringify;
    use anyhow::{anyhow, Result};
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::ResultOption;

    #[rstest]
    #[case(Ok(Some(1)), Ok(Some(2)), Ok(Some(1)))]
    #[case(Ok(None), Ok(Some(2)), Ok(Some(2)))]
    #[case(Ok(None), Ok(None), Ok(None))]
    #[case(Ok(None), Err(anyhow!("error")), Err(anyhow!("error")))]
    #[case(Err(anyhow!("error")), Ok(Some(2)), Err(anyhow!("error")))]
    fn or_if_empty_test_cases(
        #[case] v1: Result<Option<i32>>,
        #[case] v2: Result<Option<i32>>,
        #[case] expected: Result<Option<i32>>,
    ) {
        // when
        let result = v1.or_if_empty(|| v2);

        // then
        assert_eq!(result.map_err(stringify), expected.map_err(stringify));
    }

    #[rstest]
    #[case(Ok(Some(1)), Ok(2), Ok(1))]
    #[case(Ok(None), Ok(2), Ok(2))]
    #[case(Ok(None), Err(anyhow!("error")), Err(anyhow!("error")))]
    #[case(Err(anyhow!("error")), Ok(2), Err(anyhow!("error")))]
    fn otherwise_test_cases(
        #[case] v1: Result<Option<i32>>,
        #[case] v2: Result<i32>,
        #[case] expected: Result<i32>,
    ) {
        // when
        let result = v1.otherwise(|| v2);

        // then
        assert_eq!(result.map_err(stringify), expected.map_err(stringify));
    }
}
