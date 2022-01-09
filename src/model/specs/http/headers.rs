use std::{collections::HashMap, ops::Deref};

use sqlx::{
  encode::IsNull,
  error::BoxDynError,
  mysql::{MySqlTypeInfo, MySqlValueRef},
  types::Type,
  Decode, Encode, MySql,
};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpHeaders(pub HashMap<String, String>);

impl Deref for HttpHeaders {
  type Target = HashMap<String, String>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Type<MySql> for HttpHeaders {
  fn type_info() -> MySqlTypeInfo {
    <str as Type<MySql>>::type_info()
  }

  fn compatible(ty: &MySqlTypeInfo) -> bool {
    <str as Type<MySql>>::compatible(ty)
  }
}

impl Encode<'_, MySql> for HttpHeaders {
  fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
    <&str as sqlx::Encode<MySql>>::encode(&serde_json::to_string(&self).unwrap(), buf)
  }
}

impl Decode<'_, MySql> for HttpHeaders {
  fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
    Ok(serde_json::from_str(<&str as Decode<MySql>>::decode(value)?)?)
  }
}
