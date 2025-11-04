use rocket::data::Capped;
use std::fmt::Debug;

use crate::policy::FrontendPolicy;
use crate::rocket::form::{FromPConForm, PConForm};
use crate::rocket::request::PConRequest;
use crate::rocket::{PConDataField, PConValueField};
use sesame::extensions::{AsyncSesameExtension, UncheckedSesameExtension};
use sesame::pcon::PCon;
use sesame::policy::Policy;

// For multipart encoded bodies.
pub struct PConData<'a> {
    data: rocket::data::Data<'a>,
}

impl<'a> PConData<'a> {
    pub fn new(data: rocket::data::Data<'a>) -> Self {
        PConData { data }
    }
    pub fn open<'r, P: FrontendPolicy>(
        self,
        limit: rocket::data::ByteUnit,
        request: PConRequest<'a, 'r>,
    ) -> PCon<rocket::data::DataStream<'a>, P> {
        PCon::new(
            self.data.open(limit),
            P::from_request(request.get_request()),
        )
    }
    pub async fn peek<'r, P: FrontendPolicy>(
        &mut self,
        num: usize,
        request: PConRequest<'a, 'r>,
    ) -> PCon<&[u8], P> {
        let result = self.data.peek(num).await;
        PCon::new(result, P::from_request(request.get_request()))
    }
    pub fn peek_complete(&self) -> bool {
        self.data.peek_complete()
    }
    pub fn get_data(self) -> rocket::data::Data<'a> {
        self.data
    }
}

// Trait to construct stuff from data.
pub type PConDataOutcome<'a, 'r, T> = rocket::outcome::Outcome<
    T,
    (rocket::http::Status, <T as FromPConData<'a, 'r>>::PConError),
    PConData<'a>,
>;

#[rocket::async_trait]
pub trait FromPConData<'a, 'r>: Sized {
    type PConError: Send + Debug;
    async fn from_data(
        req: PConRequest<'a, 'r>,
        data: PConData<'a>,
    ) -> PConDataOutcome<'a, 'r, Self>;
}

// If T implements FromPConForm, then PConForm<T> implements FromPConData.
#[rocket::async_trait]
impl<'a, 'r, T: FromPConForm<'a, 'r>> FromPConData<'a, 'r> for PConForm<T> {
    type PConError = rocket::form::Errors<'a>;
    async fn from_data(
        req: PConRequest<'a, 'r>,
        data: PConData<'a>,
    ) -> PConDataOutcome<'a, 'r, Self> {
        use rocket::form::parser::Parser;
        use rocket::outcome::Outcome;
        use rocket::Either;
        let mut parser = match Parser::new(req.get_request(), data.get_data()).await {
            Outcome::Success(parser) => parser,
            Outcome::Failure(error) => {
                return PConDataOutcome::Failure(error);
            }
            Outcome::Forward(data) => {
                return PConDataOutcome::Forward(PConData::new(data));
            }
        };

        let mut context = T::pcon_init(rocket::form::Options::Lenient);
        while let Some(field) = parser.next().await {
            match field {
                Ok(Either::Left(value)) => {
                    T::pcon_push_value(&mut context, PConValueField::from_rocket(value), req)
                }
                Ok(Either::Right(data)) => {
                    T::pcon_push_data(&mut context, PConDataField::from_rocket(data), req).await
                }
                Err(e) => T::pcon_push_error(&mut context, e),
            }
        }

        match T::pcon_finalize(context) {
            Ok(value) => PConDataOutcome::Success(PConForm(value)),
            Err(e) => PConDataOutcome::Failure((e.status(), e)),
        }
    }
}

pub async fn into_bytes<'a, P: Policy>(
    pcon: PCon<rocket::data::DataStream<'a>, P>,
) -> std::io::Result<PCon<Capped<Vec<u8>>, P>> {
    struct Converter {}
    impl UncheckedSesameExtension for Converter {}
    #[async_trait::async_trait]
    impl<'a, P: Policy>
        AsyncSesameExtension<
            rocket::data::DataStream<'a>,
            P,
            std::io::Result<PCon<Capped<Vec<u8>>, P>>,
        > for Converter
    {
        async fn async_apply(
            &mut self,
            data: rocket::data::DataStream<'a>,
            policy: P,
        ) -> std::io::Result<PCon<Capped<Vec<u8>>, P>>
        where
            P: 'async_trait,
        {
            Ok(PCon::new(data.into_bytes().await?, policy))
        }
    }
    pcon.unchecked_async_extension(&mut Converter {}).await
}
