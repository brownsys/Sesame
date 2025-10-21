use std::fmt::{Debug, Display, Error as FmtError, Formatter};

#[derive(Clone, Debug)]
pub enum SesameError {
    PolicyCheckFailed(String),
}

impl Display for SesameError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        Debug::fmt(self, f)
    }
}
impl std::error::Error for SesameError {}

pub type SesameResult<T> = Result<T, SesameError>;
