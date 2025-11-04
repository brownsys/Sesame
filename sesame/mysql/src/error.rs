use sesame::error::SesameError;
use std::fmt::{Debug, Display, Error as FmtError, Formatter};

#[derive(Debug)]
pub enum SesameMySqlError {
    SesameError(SesameError),
    MySqlError(mysql::Error),
}

impl Display for SesameMySqlError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        Debug::fmt(self, f)
    }
}
impl std::error::Error for SesameMySqlError {}

// Conversion.
impl From<SesameError> for SesameMySqlError {
    fn from(error: SesameError) -> Self {
        SesameMySqlError::SesameError(error)
    }
}

impl From<mysql::Error> for SesameMySqlError {
    fn from(error: mysql::Error) -> Self {
        SesameMySqlError::MySqlError(error)
    }
}

// Result type.
pub type PConResult<T> = Result<T, SesameMySqlError>;
