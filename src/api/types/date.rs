use std::ops::Deref;

use chrono::NaiveDateTime;
use rocket::{http::RawStr, request::FromFormValue};

pub struct DateTime(NaiveDateTime);

impl Deref for DateTime {
  type Target = NaiveDateTime;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'v> FromFormValue<'v> for DateTime {
  type Error = &'v RawStr;

  fn from_form_value(value: &'v RawStr) -> Result<DateTime, Self::Error> {
    let decoded = value.url_decode().map_err(|_| value)?;
    let datetime = NaiveDateTime::parse_from_str(&decoded, "%Y-%m-%dT%H:%M:%S").map_err(|_| value)?;

    Ok(DateTime(datetime))
  }
}
