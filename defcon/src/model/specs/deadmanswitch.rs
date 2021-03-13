use anyhow::Result;
use sqlx::{FromRow, MySqlConnection};

use crate::model::{specs::SpecMeta, Check, Duration};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DeadManSwitch {
  #[serde(skip)]
  pub id: u64,
  #[serde(skip)]
  pub check_id: u64,
  pub stale_after: Duration,
}

impl SpecMeta for DeadManSwitch {
  fn name(&self) -> &'static str {
    "Dead man switch"
  }

  fn fields(&self) -> Vec<(&'static str, String)> {
    vec![]
  }
}

impl DeadManSwitch {
  pub async fn for_check(conn: &mut MySqlConnection, check: &Check) -> Result<DeadManSwitch> {
    let spec = sqlx::query_as::<_, DeadManSwitch>(
      "
        SELECT id, check_id, stale_after
        FROM deadmanswitch_specs
        WHERE check_id = ?
      ",
    )
    .bind(check.id)
    .fetch_one(&mut *conn)
    .await?;

    Ok(spec)
  }

  pub async fn insert(pool: &mut MySqlConnection, check: &Check, spec: DeadManSwitch) -> Result<()> {
    sqlx::query(
      "
        INSERT INTO deadmanswitch_specs ( check_id, stale_after )
        VALUES ( ?, ? )
      ",
    )
    .bind(check.id)
    .bind(spec.stale_after)
    .execute(pool)
    .await?;

    Ok(())
  }

  pub async fn update(conn: &mut MySqlConnection, check: &Check, spec: DeadManSwitch) -> Result<()> {
    sqlx::query(
      "
        UPDATE deadmanswitch_specs
        SET stale_after = ?
        WHERE check_id = ?
      ",
    )
    .bind(spec.stale_after)
    .bind(check.id)
    .execute(conn)
    .await?;

    Ok(())
  }
}
