use std::fmt::{self, Formatter};

use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
use serde::{de, ser};

use crate::model::Binary;

impl ser::Serialize for Binary {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: ser::Serializer,
  {
    serializer.serialize_str(&b64.encode(self))
  }
}

struct BinaryVisitor;

impl de::Visitor<'_> for BinaryVisitor {
  type Value = Binary;

  fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
    formatter.write_str("expecting a base64-encoded byte array")
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(Binary::from(b64.decode(value).map_err(de::Error::custom)?))
  }

  fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(Binary::from(b64.decode(value).map_err(de::Error::custom)?))
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

#[cfg(test)]
mod tests {
  use anyhow::Result;

  use super::Binary;

  #[derive(Serialize, Deserialize)]
  struct Struct {
    binary: Binary,
  }

  #[test]
  fn serialize() -> Result<()> {
    let data = Struct {
      binary: Binary::from("loremipsum".as_bytes().to_vec()),
    };

    let result = serde_json::to_string(&data)?;

    assert_eq!(&result, r#"{"binary":"bG9yZW1pcHN1bQ=="}"#);

    Ok(())
  }

  #[test]
  fn deserialize() -> Result<()> {
    let string = r#"{"binary":"bG9yZW1pcHN1bQ=="}"#;
    let result: Struct = serde_json::from_str(string)?;

    assert_eq!(std::str::from_utf8(&result.binary)?, "loremipsum");

    Ok(())
  }
}
