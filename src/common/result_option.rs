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
