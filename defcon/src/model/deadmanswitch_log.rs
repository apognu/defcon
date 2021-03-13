use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, MySqlConnection};

use crate::api::error::Shortable;

#[derive(Debug, Default, FromRow)]
pub struct DeadManSwitchLog {
  pub id: u64,
  pub check_id: u64,
  pub created_at: Option<DateTime<Utc>>,
}

impl DeadManSwitchLog {
  pub async fn last(conn: &mut MySqlConnection, check_id: u64) -> Result<Option<DeadManSwitchLog>> {
    let log = sqlx::query_as::<_, DeadManSwitchLog>(
      "
        SELECT id, check_id, created_at
        FROM deadmanswitch_logs
        WHERE check_id = ?
        ORDER BY created_at DESC
        LIMIT 1
      ",
    )
    .bind(check_id)
    .fetch_one(&mut *conn)
    .await
    .map(Some)
    .short()?;

    Ok(log)
  }

  pub async fn insert(conn: &mut MySqlConnection, check_id: u64) -> Result<()> {
    sqlx::query(
      "
        INSERT INTO deadmanswitch_logs (check_id, created_at)
        VALUES ( ?, NOW() )
      ",
    )
    .bind(check_id)
    .execute(&mut *conn)
    .await
    .short()?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use anyhow::Result;

  use super::DeadManSwitchLog;
  use crate::tests;

  #[tokio::test]
  async fn insert_and_last() -> Result<()> {
    let pool = tests::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(Some(1), None, "insert()", Some(true), None).await?;

    DeadManSwitchLog::insert(&mut conn, 1).await?;
    DeadManSwitchLog::insert(&mut conn, 1).await?;

    let check = DeadManSwitchLog::last(&mut conn, 1).await?;

    assert!(matches!(check, Some(_)));
    assert_eq!(check.unwrap().id, 1);

    pool.cleanup().await;

    Ok(())
  }
}
