use serde::{de, ser};
use std::{
  convert::TryFrom,
  fmt::{self, Formatter},
};

use crate::model::AlerterKind;

impl ser::Serialize for AlerterKind {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: ser::Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}

struct AlerterKindVisitor;

impl de::Visitor<'_> for AlerterKindVisitor {
  type Value = AlerterKind;

  fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
    formatter.write_str("a string representing an alerter kind")
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    AlerterKind::try_from(value.to_owned()).map_err(de::Error::custom)
  }

  fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    AlerterKind::try_from(value).map_err(de::Error::custom)
  }
}

impl<'de> de::Deserialize<'de> for AlerterKind {
  fn deserialize<D>(deserializer: D) -> Result<AlerterKind, D::Error>
  where
    D: de::Deserializer<'de>,
  {
    deserializer.deserialize_string(AlerterKindVisitor)
  }
}
