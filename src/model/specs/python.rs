use anyhow::Result;
use sqlx::{FromRow, MySqlConnection};

use crate::model::{specs::SpecMeta, Check};

#[derive(Debug, FromRow, Clone, Serialize, Deserialize)]
pub struct Python {
  #[serde(skip)]
  pub id: u64,
  #[serde(skip)]
  pub check_id: u64,
  pub script: String,
}

impl SpecMeta for Python {
  fn name(&self) -> &'static str {
    "External Python script"
  }

  fn fields(&self) -> Vec<(&'static str, String)> {
    vec![("Script", self.script.clone())]
  }
}

impl Python {
  pub async fn for_check(conn: &mut MySqlConnection, check: &Check) -> Result<Python> {
    let spec = sqlx::query_as::<_, Python>(
      "
        SELECT id, check_id, script
        FROM python_specs
        WHERE check_id = ?
      ",
    )
    .bind(check.id)
    .fetch_one(&mut *conn)
    .await?;

    Ok(spec)
  }

  pub async fn insert(pool: &mut MySqlConnection, check: &Check, spec: Python) -> Result<()> {
    sqlx::query(
      "
        INSERT INTO python_specs ( check_id, script )
        VALUES ( ?, ? )
      ",
    )
    .bind(check.id)
    .bind(spec.script)
    .execute(pool)
    .await?;

    Ok(())
  }

  pub async fn update(conn: &mut MySqlConnection, check: &Check, spec: Python) -> Result<()> {
    sqlx::query(
      "
        UPDATE python_specs
        SET script = ?
        WHERE check_id = ?
      ",
    )
    .bind(spec.script)
    .bind(check.id)
    .execute(conn)
    .await?;

    Ok(())
  }
}
