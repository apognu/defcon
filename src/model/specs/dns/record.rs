use std::{
  convert::TryFrom,
  fmt::{self, Display, Formatter},
};

use sqlx::{
  encode::IsNull,
  error::BoxDynError,
  mysql::{MySqlTypeInfo, MySqlValueRef},
  types::Type,
  Decode, Encode, MySql,
};
use trust_dns_client::rr::RecordType;

#[derive(Debug, Clone, PartialEq)]
pub enum DnsRecord {
  NS,
  MX,
  A,
  AAAA,
  CNAME,
  CAA,
  SRV,
}

impl Display for DnsRecord {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
    use DnsRecord::*;

    let name = match self {
      NS => "NS",
      MX => "MX",
      A => "A",
      AAAA => "AAAA",
      CNAME => "CNAME",
      CAA => "CAA",
      SRV => "SRV",
    };

    write!(formatter, "{}", name)
  }
}

impl Default for DnsRecord {
  fn default() -> DnsRecord {
    DnsRecord::A
  }
}

impl From<DnsRecord> for RecordType {
  fn from(record_type: DnsRecord) -> RecordType {
    use DnsRecord::*;

    match record_type {
      NS => RecordType::NS,
      MX => RecordType::MX,
      A => RecordType::A,
      AAAA => RecordType::AAAA,
      CNAME => RecordType::CNAME,
      CAA => RecordType::CAA,
      SRV => RecordType::SRV,
    }
  }
}

impl TryFrom<String> for DnsRecord {
  type Error = anyhow::Error;

  fn try_from(record_type: String) -> Result<DnsRecord, Self::Error> {
    use DnsRecord::*;

    match record_type.as_str() {
      "NS" => Ok(NS),
      "MX" => Ok(MX),
      "A" => Ok(A),
      "AAAA" => Ok(AAAA),
      "CNAME" => Ok(CNAME),
      "CAA" => Ok(CAA),
      "SRV" => Ok(SRV),
      _ => Err(anyhow!("invalid value for record_type")),
    }
  }
}

impl Type<MySql> for DnsRecord {
  fn type_info() -> MySqlTypeInfo {
    <str as Type<MySql>>::type_info()
  }

  fn compatible(ty: &MySqlTypeInfo) -> bool {
    <str as Type<MySql>>::compatible(ty)
  }
}

impl Encode<'_, MySql> for DnsRecord {
  fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
    <&str as sqlx::Encode<MySql>>::encode(&self.to_string(), buf)
  }
}

impl Decode<'_, MySql> for DnsRecord {
  fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
    Ok(DnsRecord::try_from(<&str as Decode<MySql>>::decode(value).map(ToOwned::to_owned)?)?)
  }
}
