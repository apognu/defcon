use std::{
  convert::TryFrom,
  fmt::{self, Formatter},
};

use crate::model::specs::DnsRecord;

use serde::{de, ser};

impl ser::Serialize for DnsRecord {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: ser::Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}

struct DnsRecordVisitor;

impl de::Visitor<'_> for DnsRecordVisitor {
  type Value = DnsRecord;

  fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
    formatter.write_str("a string representing a DNS record type")
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    DnsRecord::try_from(value.to_owned()).map_err(de::Error::custom)
  }

  fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    DnsRecord::try_from(value).map_err(de::Error::custom)
  }
}

impl<'de> de::Deserialize<'de> for DnsRecord {
  fn deserialize<D>(deserializer: D) -> Result<DnsRecord, D::Error>
  where
    D: de::Deserializer<'de>,
  {
    deserializer.deserialize_string(DnsRecordVisitor)
  }
}
