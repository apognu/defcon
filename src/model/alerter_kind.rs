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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AlerterKind {
  Webhook,
  Slack,
  Pagerduty,
  Noop,
}

impl Default for AlerterKind {
  fn default() -> AlerterKind {
    AlerterKind::Noop
  }
}

impl Display for AlerterKind {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
    let name = match self {
      AlerterKind::Webhook => "webhook",
      AlerterKind::Slack => "slack",
      AlerterKind::Pagerduty => "pagerduty",
      AlerterKind::Noop => "noop",
    };

    write!(formatter, "{}", name)
  }
}

impl TryFrom<String> for AlerterKind {
  type Error = anyhow::Error;

  fn try_from(kind: String) -> Result<AlerterKind, Self::Error> {
    match kind.as_str() {
      "webhook" => Ok(AlerterKind::Webhook),
      "slack" => Ok(AlerterKind::Slack),
      "pagerduty" => Ok(AlerterKind::Pagerduty),
      "noop" => Ok(AlerterKind::Noop),
      _ => Err(anyhow!("unknown alerter kind")),
    }
  }
}

impl Type<MySql> for AlerterKind {
  fn type_info() -> MySqlTypeInfo {
    <str as Type<MySql>>::type_info()
  }

  fn compatible(ty: &MySqlTypeInfo) -> bool {
    <str as Type<MySql>>::compatible(ty)
  }
}

impl Encode<'_, MySql> for AlerterKind {
  fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
    <&str as sqlx::Encode<MySql>>::encode(&self.to_string(), buf)
  }
}

impl Decode<'_, MySql> for AlerterKind {
  fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
    Ok(AlerterKind::try_from(<&str as Decode<MySql>>::decode(value).map(ToOwned::to_owned)?)?)
  }
}
