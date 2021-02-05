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

#[derive(Debug, PartialEq)]
pub enum CheckKind {
  Ping,
  Dns,
  Http,
  Tcp,
  Udp,
  Tls,
  PlayStore,
  AppStore,
  Whois,
}

impl Default for CheckKind {
  fn default() -> CheckKind {
    CheckKind::Ping
  }
}

impl Display for CheckKind {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
    use CheckKind::*;

    let name = match self {
      Ping => "ping",
      Dns => "dns",
      Http => "http",
      Tcp => "tcp",
      Udp => "udp",
      Tls => "tls",
      PlayStore => "play_store",
      AppStore => "app_store",
      Whois => "domain",
    };

    write!(formatter, "{}", name.to_string())
  }
}

impl TryFrom<String> for CheckKind {
  type Error = anyhow::Error;

  fn try_from(kind: String) -> Result<CheckKind, Self::Error> {
    use CheckKind::*;

    match kind.as_str() {
      "ping" => Ok(Ping),
      "dns" => Ok(Dns),
      "http" => Ok(Http),
      "tcp" => Ok(Tcp),
      "udp" => Ok(Udp),
      "tls" => Ok(Tls),
      "play_store" => Ok(PlayStore),
      "app_store" => Ok(AppStore),
      "domain" => Ok(Whois),
      _ => Err(anyhow!("invalid value for kind")),
    }
  }
}

impl Type<MySql> for CheckKind {
  fn type_info() -> MySqlTypeInfo {
    <str as Type<MySql>>::type_info()
  }

  fn compatible(ty: &MySqlTypeInfo) -> bool {
    <str as Type<MySql>>::compatible(ty)
  }
}

impl Encode<'_, MySql> for CheckKind {
  fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
    <&str as sqlx::Encode<MySql>>::encode(&self.to_string(), buf)
  }
}

impl Decode<'_, MySql> for CheckKind {
  fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
    Ok(CheckKind::try_from(<&str as Decode<MySql>>::decode(value).map(ToOwned::to_owned)?)?)
  }
}