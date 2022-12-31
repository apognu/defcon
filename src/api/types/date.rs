use std::{fmt, ops::Deref};

use chrono::{NaiveDate, NaiveDateTime};
use serde::de::{self, Deserialize, Deserializer, Visitor};

pub struct DateTime(NaiveDateTime);

impl Deref for DateTime {
  type Target = NaiveDateTime;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

struct DateTimeVisitor;

impl<'de> Visitor<'de> for DateTimeVisitor {
  type Value = DateTime;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a date in YYYY-MM-DDTHH:mm:SS format")
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    let datetime = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S").map_err(|_| de::Error::custom("invalid date"))?;

    Ok(DateTime(datetime))
  }
}

impl<'de> Deserialize<'de> for DateTime {
  fn deserialize<D>(deserializer: D) -> Result<DateTime, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_str(DateTimeVisitor)
  }
}

pub struct Date(NaiveDate);

impl Deref for Date {
  type Target = NaiveDate;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

struct DateVisitor;

impl<'de> Visitor<'de> for DateVisitor {
  type Value = Date;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a date in YYYY-MM-DD format")
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| de::Error::custom("invalid date"))?;

    Ok(Date(date))
  }
}

impl<'de> Deserialize<'de> for Date {
  fn deserialize<D>(deserializer: D) -> Result<Date, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_str(DateVisitor)
  }
}
