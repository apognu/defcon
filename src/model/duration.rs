use std::{
  fmt::{self, Formatter},
  ops::Deref,
  time::Duration as StdDuration,
};

use serde::{de, ser};
use sqlx::{
  encode::IsNull,
  error::BoxDynError,
  mysql::{MySqlTypeInfo, MySqlValueRef},
  types::Type,
  Decode, Encode, MySql,
};

#[derive(Debug, Default)]
pub struct Duration(pub StdDuration);

impl Deref for Duration {
  type Target = StdDuration;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl AsRef<StdDuration> for Duration {
  fn as_ref(&self) -> &StdDuration {
    &self.0
  }
}

impl From<u32> for Duration {
  fn from(seconds: u32) -> Duration {
    Duration(StdDuration::from_secs(seconds as u64))
  }
}

impl From<u64> for Duration {
  fn from(seconds: u64) -> Duration {
    Duration(StdDuration::from_secs(seconds))
  }
}

impl Type<MySql> for Duration {
  fn type_info() -> MySqlTypeInfo {
    <u32 as Type<MySql>>::type_info()
  }

  fn compatible(ty: &MySqlTypeInfo) -> bool {
    <u32 as Type<MySql>>::compatible(ty)
  }
}

impl Encode<'_, MySql> for Duration {
  fn encode(self, buf: &mut Vec<u8>) -> IsNull
  where
    Self: Sized,
  {
    <u32 as sqlx::Encode<MySql>>::encode(self.as_secs() as u32, buf)
  }

  fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
    <&u32 as sqlx::Encode<MySql>>::encode(&(self.as_secs() as u32), buf)
  }
}

impl Decode<'_, MySql> for Duration {
  fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
    Ok(Duration(StdDuration::from_secs(<u32 as Decode<MySql>>::decode(value)? as u64)))
  }
}

impl ser::Serialize for Duration {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: ser::Serializer,
  {
    serializer.serialize_u64(self.as_secs())
  }
}

struct DurationVisitor;

impl<'de> de::Visitor<'de> for DurationVisitor {
  type Value = Duration;

  fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
    formatter.write_str("a number of seconds representing a duration")
  }

  fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(Duration::from(value))
  }
}

impl<'de> de::Deserialize<'de> for Duration {
  fn deserialize<D>(deserializer: D) -> Result<Duration, D::Error>
  where
    D: de::Deserializer<'de>,
  {
    deserializer.deserialize_u64(DurationVisitor)
  }
}
