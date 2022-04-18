use anyhow::Result;
use sqlx::{FromRow, MySqlConnection};

use crate::model::{specs::SpecMeta, Check};

pub use super::record::*;

#[derive(Debug, FromRow, Clone, Serialize, Deserialize)]
pub struct Dns {
  #[serde(skip)]
  pub id: u64,
  #[serde(skip)]
  pub check_id: u64,
  #[serde(default)]
  pub record: DnsRecord,
  pub domain: String,
  pub value: String,
}

impl SpecMeta for Dns {
  fn name(&self) -> &'static str {
    "DNS resolution"
  }

  fn fields(&self) -> Vec<(&'static str, String)> {
    vec![("Record type", self.record.to_string()), ("Domain", self.domain.clone())]
  }
}

impl Dns {
  pub async fn for_check(conn: &mut MySqlConnection, check: &Check) -> Result<Dns> {
    let spec = sqlx::query_as::<_, Dns>(
      "
        SELECT id, check_id, record, domain, value
        FROM dns_specs
        WHERE check_id = ?
      ",
    )
    .bind(check.id)
    .fetch_one(&mut *conn)
    .await?;

    Ok(spec)
  }

  pub async fn insert(pool: &mut MySqlConnection, check: &Check, spec: Dns) -> Result<()> {
    sqlx::query(
      "
        INSERT INTO dns_specs ( check_id, record, domain, value )
        VALUES ( ?, ?, ?, ? )
      ",
    )
    .bind(check.id)
    .bind(spec.record)
    .bind(spec.domain)
    .bind(spec.value)
    .execute(pool)
    .await?;

    Ok(())
  }

  pub async fn update(conn: &mut MySqlConnection, check: &Check, spec: Dns) -> Result<()> {
    sqlx::query(
      "
        UPDATE dns_specs
        SET record = ?, domain = ?, value = ?
        WHERE check_id = ?
      ",
    )
    .bind(spec.record)
    .bind(spec.domain)
    .bind(spec.value)
    .bind(check.id)
    .execute(conn)
    .await?;

    Ok(())
  }
}
