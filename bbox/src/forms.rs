extern crate rocket;

use rocket::form::{self, FromFormField, DataField, ValueField};
use crate::BBox;

// Allows us to load structs with BBox fields from rocket forms automatically.
#[rocket::async_trait]
impl<'r, T> FromFormField<'r> for BBox<T> where T: Send + Clone {
    fn from_value(_field: ValueField<'r>) -> form::Result<'r, Self> {
        todo!("parse from a value or use default impl")
    }

    async fn from_data(_field: DataField<'r, '_>) -> form::Result<'r, Self> {
        todo!("parse from a value or use default impl")
    }
}
