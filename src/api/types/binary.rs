use std::fmt::{self, Formatter};

use serde::{de, ser};

use crate::model::Binary;

impl ser::Serialize for Binary {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: ser::Serializer,
  {
    serializer.serialize_str(&base64::encode(self))
  }
}

struct BinaryVisitor;

impl<'de> de::Visitor<'de> for BinaryVisitor {
  type Value = Binary;

  fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
    formatter.write_str("expecting a base64-encoded byte array")
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(Binary::from(base64::decode(value).map_err(de::Error::custom)?))
  }

  fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(Binary::from(base64::decode(&value).map_err(de::Error::custom)?))
  }
}

impl<'de> de::Deserialize<'de> for Binary {
  fn deserialize<D>(deserializer: D) -> Result<Binary, D::Error>
  where
    D: de::Deserializer<'de>,
  {
    deserializer.deserialize_string(BinaryVisitor)
  }
}
