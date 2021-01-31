use std::{
  fmt::{self, Display, Formatter},
  ops::Deref,
};

use sqlx::{
  encode::IsNull,
  error::BoxDynError,
  mysql::{MySqlTypeInfo, MySqlValueRef},
  types::Type,
  Decode, Encode, MySql,
};

#[derive(Debug)]
pub struct Binary(Vec<u8>);

impl Display for Binary {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
    write!(formatter, "<binary>")
  }
}

impl Deref for Binary {
  type Target = Vec<u8>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl AsRef<[u8]> for Binary {
  fn as_ref(&self) -> &[u8] {
    &self.0
  }
}

impl From<Vec<u8>> for Binary {
  fn from(bytes: Vec<u8>) -> Binary {
    Binary(bytes)
  }
}

impl Type<MySql> for Binary {
  fn type_info() -> MySqlTypeInfo {
    <Vec<u8> as Type<MySql>>::type_info()
  }

  fn compatible(ty: &MySqlTypeInfo) -> bool {
    <Vec<u8> as Type<MySql>>::compatible(ty)
  }
}

impl Encode<'_, MySql> for Binary {
  fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
    <&Vec<u8> as sqlx::Encode<MySql>>::encode(self, buf)
  }
}

impl Decode<'_, MySql> for Binary {
  fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
    Ok(Binary::from(<Vec<u8> as Decode<MySql>>::decode(value)?))
  }
}
