use std::fmt::Debug;

use rocket::data::Capped;

use crate::bbox::BBox;
use crate::policy::{FrontendPolicy, Policy};
use crate::rocket::{BBoxDataField, BBoxValueField};
use crate::rocket::form::{BBoxForm, FromBBoxForm};
use crate::rocket::request::BBoxRequest;

// For multipart encoded bodies.
pub struct BBoxData<'a> {
    data: rocket::data::Data<'a>,
}

impl<'a> BBoxData<'a> {
    pub fn new(data: rocket::data::Data<'a>) -> Self {
        BBoxData { data }
    }
    pub fn open<'r, P: FrontendPolicy>(
        self,
        limit: rocket::data::ByteUnit,
        request: BBoxRequest<'a, 'r>,
    ) -> BBox<rocket::data::DataStream<'a>, P> {
        BBox::new(self.data.open(limit), P::from_request(request.get_request()))
    }
    pub async fn peek<'r, P: FrontendPolicy>(
        &mut self,
        num: usize,
        request: BBoxRequest<'a, 'r>,
    ) -> BBox<&[u8], P> {
        let result = self.data.peek(num).await;
        BBox::new(result, P::from_request(request.get_request()))
    }
    pub fn peek_complete(&self) -> bool {
        self.data.peek_complete()
    }
    pub fn get_data(self) -> rocket::data::Data<'a> {
        self.data
    }
}

// Trait to construct stuff from data.
pub type BBoxDataOutcome<'a, 'r, T> =
    rocket::outcome::Outcome<T, (rocket::http::Status, <T as FromBBoxData<'a, 'r>>::BBoxError), BBoxData<'a>>;

#[rocket::async_trait]
pub trait FromBBoxData<'a, 'r>: Sized {
    type BBoxError: Send + Debug;
    async fn from_data(
        req: BBoxRequest<'a, 'r>,
        data: BBoxData<'a>,
    ) -> BBoxDataOutcome<'a, 'r, Self>;
}

// If T implements FromBBoxForm, then BBoxForm<T> implements FromBBoxData.
#[rocket::async_trait]
impl<'a, 'r, T: FromBBoxForm<'a, 'r>> FromBBoxData<'a, 'r> for BBoxForm<T>{
    type BBoxError = rocket::form::Errors<'a>;
    async fn from_data(
        req: BBoxRequest<'a, 'r>,
        data: BBoxData<'a>,
    ) -> BBoxDataOutcome<'a, 'r, Self> {
        use rocket::Either;
        use rocket::outcome::Outcome;
        use rocket::form::parser::Parser;
        let mut parser = match Parser::new(req.get_request(), data.get_data()).await {
            Outcome::Success(parser) => parser,
            Outcome::Failure(error) => {
                return BBoxDataOutcome::Failure(error);
            },
            Outcome::Forward(data) => {
                return BBoxDataOutcome::Forward(BBoxData::new(data));
            },
        };

        let mut context = T::bbox_init(rocket::form::Options::Lenient);
        while let Some(field) = parser.next().await {
            match field {
                Ok(Either::Left(value)) => {
                    T::bbox_push_value(&mut context, BBoxValueField::from_rocket(value), req)
                },
                Ok(Either::Right(data)) => {
                    T::bbox_push_data(&mut context, BBoxDataField::from_rocket(data), req).await
                },
                Err(e) => T::bbox_push_error(&mut context, e),
            }
        }

        match T::bbox_finalize(context) {
            Ok(value) => BBoxDataOutcome::Success(BBoxForm(value)),
            Err(e) => BBoxDataOutcome::Failure((e.status(), e)),
        }
    }
}

impl<'a, P: Policy> BBox<rocket::data::DataStream<'a>, P> {
    pub async fn into_bytes(self) -> std::io::Result<BBox<Capped<Vec<u8>>, P>> {
        let (t, p) = self.consume();
        Ok(BBox::new(t.into_bytes().await?, p))
    }
}
