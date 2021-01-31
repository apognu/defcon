use std::{
  convert::TryFrom,
  fmt::{self, Formatter},
};

use humantime::format_duration;
use serde::{de, ser};

use crate::model::Duration;

impl ser::Serialize for Duration {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: ser::Serializer,
  {
    serializer.serialize_str(&format_duration(self.0).to_string())
  }
}

struct DurationVisitor;

impl<'de> de::Visitor<'de> for DurationVisitor {
  type Value = Duration;

  fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
    formatter.write_str("a human-friendly string representing a duration")
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Duration::try_from(value.to_string()).map_err(de::Error::custom)
  }

  fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Duration::try_from(value).map_err(de::Error::custom)
  }
}

impl<'de> de::Deserialize<'de> for Duration {
  fn deserialize<D>(deserializer: D) -> Result<Duration, D::Error>
  where
    D: de::Deserializer<'de>,
  {
    deserializer.deserialize_str(DurationVisitor)
  }
}
