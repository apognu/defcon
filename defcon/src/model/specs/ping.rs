use anyhow::Result;
use sqlx::{FromRow, MySqlConnection};

use crate::model::{specs::SpecMeta, Check};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Ping {
  #[serde(skip)]
  pub id: u64,
  #[serde(skip)]
  pub check_id: u64,
  pub host: String,
}

impl SpecMeta for Ping {
  fn name(&self) -> &'static str {
    "Echo request"
  }

  fn fields(&self) -> Vec<(&'static str, String)> {
    vec![("Host", self.host.clone())]
  }
}

impl Ping {
  pub async fn for_check(conn: &mut MySqlConnection, check: &Check) -> Result<Ping> {
    let spec = sqlx::query_as::<_, Ping>(
      "
        SELECT id, check_id, host
        FROM ping_specs
        WHERE check_id = ?
      ",
    )
    .bind(check.id)
    .fetch_one(&mut *conn)
    .await?;

    Ok(spec)
  }

  pub async fn insert(pool: &mut MySqlConnection, check: &Check, spec: Ping) -> Result<()> {
    sqlx::query(
      "
        INSERT INTO ping_specs ( check_id, host )
        VALUES ( ?, ? )
      ",
    )
    .bind(check.id)
    .bind(spec.host)
    .execute(pool)
    .await?;

    Ok(())
  }

  pub async fn update(conn: &mut MySqlConnection, check: &Check, spec: Ping) -> Result<()> {
    sqlx::query(
      "
        UPDATE ping_specs
        SET host = ?
        WHERE check_id = ?
      ",
    )
    .bind(spec.host)
    .bind(check.id)
    .execute(conn)
    .await?;

    Ok(())
  }
}
