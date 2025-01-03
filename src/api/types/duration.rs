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

impl de::Visitor<'_> for DurationVisitor {
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

#[cfg(test)]
mod tests {
  use anyhow::Result;

  use super::Duration;

  #[derive(Serialize, Deserialize)]
  struct Struct {
    duration: Duration,
  }

  #[test]
  fn serialize() -> Result<()> {
    let data = Struct { duration: Duration::from(3600) };
    let result = serde_json::to_string(&data)?;

    assert_eq!(&result, r#"{"duration":"1h"}"#);

    Ok(())
  }

  #[test]
  fn deserialize() -> Result<()> {
    let string = r#"{"duration":"1h"}"#;
    let result: Struct = serde_json::from_str(string)?;

    assert_eq!(*result.duration, *Duration::from(3600));

    Ok(())
  }
}
