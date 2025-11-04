use crate::rocket::{PConRequest, PConResponder, PConResponseResult};
use sesame::error::SesameError;
use std::fmt::{Debug, Display, Formatter};

// All errors should implement this.
impl<'a, 'r, 'o: 'r> PConResponder<'a, 'r, 'o> for SesameError {
    fn respond_to(self, _request: PConRequest<'a, 'r>) -> PConResponseResult<'o> {
        Err(rocket::http::Status { code: 491 })
    }
}

#[cfg(feature = "mysql")]
mod mysql {
    use crate::rocket::{PConRequest, PConResponder, PConResponseResult};
    use sesame_mysql::SesameMySqlError;

    impl<'a, 'r, 'o: 'r> PConResponder<'a, 'r, 'o> for SesameMySqlError {
        fn respond_to(self, request: PConRequest<'a, 'r>) -> PConResponseResult<'o> {
            match self {
                SesameMySqlError::SesameError(error) => error.respond_to(request),
                SesameMySqlError::MySqlError(_error) => Err(rocket::http::Status { code: 500 }),
            }
        }
    }
}

// Errors that can occur during rendering.
#[derive(Clone, Debug)]
pub enum SesameRenderError {
    SesameError(SesameError),
    FigmentError(figment::Error),
}
impl Display for SesameRenderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}
impl std::error::Error for SesameRenderError {}
impl<'a, 'r, 'o: 'r> PConResponder<'a, 'r, 'o> for SesameRenderError {
    fn respond_to(self, request: PConRequest<'a, 'r>) -> PConResponseResult<'o> {
        match self {
            SesameRenderError::SesameError(err) => err.respond_to(request),
            SesameRenderError::FigmentError(_err) => Err(rocket::http::Status::InternalServerError),
        }
    }
}

// Error conversion.
impl From<SesameError> for SesameRenderError {
    fn from(e: SesameError) -> Self {
        SesameRenderError::SesameError(e)
    }
}
impl From<figment::Error> for SesameRenderError {
    fn from(e: figment::Error) -> Self {
        SesameRenderError::FigmentError(e)
    }
}

// Results.
pub type SesameRenderResult<T> = Result<T, SesameRenderError>;
