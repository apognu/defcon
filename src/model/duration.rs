use std::{convert::TryFrom, ops::Deref, time::Duration as StdDuration};

use humantime::parse_duration;
use sqlx::{
  encode::IsNull,
  error::BoxDynError,
  mysql::{MySqlTypeInfo, MySqlValueRef},
  types::Type,
  Decode, Encode, MySql,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct Duration(pub StdDuration);

impl Deref for Duration {
  type Target = StdDuration;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl TryFrom<String> for Duration {
  type Error = anyhow::Error;

  fn try_from(value: String) -> Result<Duration, Self::Error> {
    Ok(Duration(parse_duration(&value)?))
  }
}

impl From<u64> for Duration {
  fn from(seconds: u64) -> Duration {
    Duration(StdDuration::from_secs(seconds))
  }
}

impl Type<MySql> for Duration {
  fn type_info() -> MySqlTypeInfo {
    <u64 as Type<MySql>>::type_info()
  }

  fn compatible(ty: &MySqlTypeInfo) -> bool {
    <u64 as Type<MySql>>::compatible(ty)
  }
}

impl Encode<'_, MySql> for Duration {
  fn encode(self, buf: &mut Vec<u8>) -> IsNull
  where
    Self: Sized,
  {
    <u64 as sqlx::Encode<MySql>>::encode(self.as_secs(), buf)
  }

  fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
    <&u64 as sqlx::Encode<MySql>>::encode(&self.as_secs(), buf)
  }
}

impl Decode<'_, MySql> for Duration {
  fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
    Ok(Duration(StdDuration::from_secs(<u64 as Decode<MySql>>::decode(value)?)))
  }
}
