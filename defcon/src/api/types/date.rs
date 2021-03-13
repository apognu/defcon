use std::ops::Deref;

use chrono::NaiveDateTime;
use rocket::form::{error::ErrorKind, FromFormField, Result as FormResult, ValueField};

pub struct DateTime(NaiveDateTime);

impl Deref for DateTime {
  type Target = NaiveDateTime;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'v> FromFormField<'v> for DateTime {
  fn from_value(field: ValueField<'v>) -> FormResult<'v, DateTime> {
    let datetime = NaiveDateTime::parse_from_str(field.value, "%Y-%m-%dT%H:%M:%S").map_err(|_| ErrorKind::Unknown)?;

    Ok(DateTime(datetime))
  }
}
