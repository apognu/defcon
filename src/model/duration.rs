use std::{convert::TryFrom, error::Error, ops::Deref, time::Duration as StdDuration};

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

impl From<u64> for Duration {
  fn from(seconds: u64) -> Duration {
    Duration(StdDuration::from_secs(seconds))
  }
}

impl TryFrom<&str> for Duration {
  type Error = anyhow::Error;

  fn try_from(value: &str) -> Result<Duration, Self::Error> {
    Ok(Duration(parse_duration(value)?))
  }
}

impl TryFrom<String> for Duration {
  type Error = anyhow::Error;

  fn try_from(value: String) -> Result<Duration, Self::Error> {
    Duration::try_from(value.as_ref())
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
  fn encode(self, buf: &mut Vec<u8>) -> Result<IsNull, Box<dyn Error + Send + Sync + 'static>>
  where
    Self: Sized,
  {
    <u64 as sqlx::Encode<MySql>>::encode(self.as_secs(), buf)
  }

  fn encode_by_ref(&self, buf: &mut Vec<u8>) -> Result<IsNull, Box<dyn Error + Send + Sync + 'static>> {
    <u64 as sqlx::Encode<MySql>>::encode(self.as_secs(), buf)
  }
}

impl Decode<'_, MySql> for Duration {
  fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
    Ok(Duration(StdDuration::from_secs(<u64 as Decode<MySql>>::decode(value)?)))
  }
}

#[cfg(test)]
mod tests {
  use std::convert::TryFrom;

  use super::Duration;

  #[test]
  fn try_from_str() {
    assert_eq!(Duration::try_from("10s").is_ok(), true);
    assert_eq!(Duration::try_from("10s").unwrap().as_secs(), 10);

    assert_eq!(Duration::try_from("10m").is_ok(), true);
    assert_eq!(Duration::try_from("10m").unwrap().as_secs(), 600);

    assert_eq!(Duration::try_from("2d").is_ok(), true);
    assert_eq!(Duration::try_from("2d").unwrap().as_secs(), 172800);
  }

  #[test]
  fn from_u64() {
    assert_eq!(Duration::from(10).as_secs(), 10);
    assert_eq!(Duration::from(300).as_secs(), 300);
    assert_eq!(Duration::from(172800).as_secs(), 172800);
  }
}
