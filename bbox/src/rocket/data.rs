use std::fmt::Debug;

use crate::bbox::BBox;
use crate::policy::{DefaultConstructablePolicy};
use crate::rocket::form::{BBoxForm, FromBBoxForm};
use crate::rocket::request::{BBoxRequest, FromFormWrapper};
use crate::rocket::TmpPolicy;

// For multipart encoded bodies.
pub struct BBoxData<'r> {
    data: rocket::data::Data<'r>,
}

impl<'r> BBoxData<'r> {
    pub fn new(data: rocket::data::Data<'r>) -> Self {
        BBoxData { data }
    }
    pub fn open(self, limit: rocket::data::ByteUnit) -> BBox<rocket::data::DataStream<'r>, TmpPolicy> {
        BBox::new(self.data.open(limit), TmpPolicy::construct())
    }
    pub async fn peek(&mut self, num: usize) -> BBox<&[u8], TmpPolicy> {
        let result = self.data.peek(num).await;
        BBox::new(result, TmpPolicy::construct())
    }
    pub fn peek_complete(&self) -> bool {
        self.data.peek_complete()
    }
    pub(crate) fn get_data(self) -> rocket::data::Data<'r> {
        self.data
    }
}

// Trait to construct stuff from data.
pub type BBoxDataOutcome<'r, T, E = <T as FromBBoxData<'r>>::BBoxError> =
    rocket::outcome::Outcome<T, (rocket::http::Status, E), BBoxData<'r>>;

#[rocket::async_trait]
pub trait FromBBoxData<'r>: Sized {
    type BBoxError: Send + Debug;
    async fn from_data(req: &BBoxRequest<'r, '_>, data: BBoxData<'r>) -> BBoxDataOutcome<'r, Self>;
}

// If T implements FromBBoxForm, then BBoxForm<T> implements FromBBoxData.
#[rocket::async_trait]
impl<'r, T: FromBBoxForm<'r>> FromBBoxData<'r> for BBoxForm<T> {
    type BBoxError = rocket::form::Errors<'r>;
    async fn from_data(req: &BBoxRequest<'r, '_>, data: BBoxData<'r>) -> BBoxDataOutcome<'r, Self> {
        use rocket::data::FromData;
        match rocket::form::Form::<FromFormWrapper<T>>::from_data(
            req.get_request(),
            data.get_data(),
        )
        .await
        {
            rocket::outcome::Outcome::Success(form) => {
                BBoxDataOutcome::Success(BBoxForm(form.into_inner().0))
            }
            rocket::outcome::Outcome::Failure(error) => BBoxDataOutcome::Failure(error),
            rocket::outcome::Outcome::Forward(data) => {
                BBoxDataOutcome::Forward(BBoxData::new(data))
            }
        }
    }
}
