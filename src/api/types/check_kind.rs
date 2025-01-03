use std::{
  convert::TryFrom,
  fmt::{self, Formatter},
};

use crate::model::CheckKind;

use serde::{de, ser};

impl ser::Serialize for CheckKind {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: ser::Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}

struct CheckKindVisitor;

impl de::Visitor<'_> for CheckKindVisitor {
  type Value = CheckKind;

  fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
    formatter.write_str("a string representing a check kind")
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    CheckKind::try_from(value.to_owned()).map_err(de::Error::custom)
  }

  fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    CheckKind::try_from(value).map_err(de::Error::custom)
  }
}

impl<'de> de::Deserialize<'de> for CheckKind {
  fn deserialize<D>(deserializer: D) -> Result<CheckKind, D::Error>
  where
    D: de::Deserializer<'de>,
  {
    deserializer.deserialize_string(CheckKindVisitor)
  }
}
