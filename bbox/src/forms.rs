extern crate rocket;

use std::str::FromStr;
use rocket::form::{self, FromFormField, DataField, ValueField, Errors};
use rocket::form::error::ErrorKind;
use crate::BBox;

// Allows us to load structs with BBox fields from rocket forms automatically.
#[rocket::async_trait]
impl<'r, T> FromFormField<'r> for BBox<T> where T: Send + Clone + FromStr {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        match field.value.parse::<T>() {
            Ok(converted) => Ok(BBox::new(converted)),
            Err(_) => Err(Errors::from(ErrorKind::Unexpected))
        }
    }

    async fn from_data(_field: DataField<'r, '_>) -> form::Result<'r, Self> {
        todo!("parse from a value or use default impl")
    }
}

// Facilitate URL parameter conversion.
impl<'r, T: FromStr> FromParam<'r> for BBox<T> {
  type Error = &'r str;

  fn from_param(param: &'r str) -> Result<Self, Self::Error> {
    match param.parse::<T>() {
      Ok(converted) => Ok(BBox::new(converted)),
      Err(_) => Err(param)
    }
  }
}
