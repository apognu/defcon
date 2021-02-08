use anyhow::Result;
use sqlx::{FromRow, MySqlConnection};

use crate::model::{specs::SpecMeta, Check, Duration};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Tls {
  #[serde(skip)]
  pub id: u64,
  #[serde(skip)]
  pub check_id: u64,
  pub domain: String,
  pub window: Duration,
}

impl SpecMeta for Tls {
  fn name(&self) -> &'static str {
    "TLS certificate expiration"
  }

  fn fields(&self) -> Vec<(&'static str, String)> {
    vec![("Domain", self.domain.clone())]
  }
}

impl Tls {
  pub async fn for_check(conn: &mut MySqlConnection, check: &Check) -> Result<Tls> {
    let spec = sqlx::query_as::<_, Tls>(
      "
        SELECT id, check_id, domain, window
        FROM tls_specs
        WHERE check_id = ?
      ",
    )
    .bind(check.id)
    .fetch_one(&mut *conn)
    .await?;

    Ok(spec)
  }

  pub async fn insert(pool: &mut MySqlConnection, check: &Check, spec: Tls) -> Result<()> {
    sqlx::query(
      "
        INSERT INTO tls_specs ( check_id, domain, window )
        VALUES ( ?, ?, ? )
      ",
    )
    .bind(check.id)
    .bind(spec.domain)
    .bind(spec.window)
    .execute(pool)
    .await?;

    Ok(())
  }

  pub async fn update(conn: &mut MySqlConnection, check: &Check, spec: Tls) -> Result<()> {
    sqlx::query(
      "
        UPDATE tls_specs
        SET domain = ?, window = ?
        WHERE check_id = ?
      ",
    )
    .bind(spec.domain)
    .bind(spec.window)
    .bind(check.id)
    .execute(conn)
    .await?;

    Ok(())
  }
}
