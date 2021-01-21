use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::{Done, Executor, FromRow, MySql, MySqlConnection};

use crate::model::Outage;

pub mod status {
  pub const OK: u8 = 0;
  pub const CRITICAL: u8 = 1;
  pub const WARNING: u8 = 2;
}

#[derive(Debug, Default, FromRow, Serialize)]
pub struct Event {
  #[serde(skip_serializing)]
  pub id: u64,
  #[serde(skip_serializing)]
  pub check_id: u64,
  #[serde(skip_serializing)]
  pub outage_id: Option<u64>,
  pub status: u8,
  pub message: String,
  pub created_at: Option<DateTime<Utc>>,
}

impl Event {
  pub async fn for_outage(conn: &mut MySqlConnection, outage: &Outage) -> Result<Vec<Event>> {
    let events = sqlx::query_as::<_, Event>(
      "
        SELECT id, check_id, outage_id, status, message, created_at
        FROM events
        WHERE outage_id = ?
      ",
    )
    .bind(outage.id)
    .fetch_all(&mut *conn)
    .await?;

    Ok(events)
  }

  pub async fn insert(conn: &mut MySqlConnection, event: &Event, outage: Option<&Outage>) -> Result<()> {
    sqlx::query(
      "
        INSERT INTO events (check_id, outage_id, status, message, created_at)
        VALUES ( ?, ?, ?, ?, NOW() )
      ",
    )
    .bind(event.check_id)
    .bind(outage.map(|outage| outage.id))
    .bind(event.status)
    .bind(&event.message)
    .execute(conn)
    .await?;

    Ok(())
  }

  pub async fn delete_before<'c, E>(pool: E, epoch: &NaiveDateTime) -> Result<u64>
  where
    E: Executor<'c, Database = MySql>,
  {
    let result = sqlx::query(
      "
        DELETE events FROM events
        LEFT JOIN outages
        ON outages.id = events.outage_id
        WHERE ended_on IS NOT NULL AND ended_on < ?
      ",
    )
    .bind(epoch)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
  }
}
