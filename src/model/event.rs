use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::{Done, FromRow, MySqlConnection};

use crate::model::SiteOutage;

pub mod status {
  pub const OK: u8 = 0;
  pub const CRITICAL: u8 = 1;
  pub const WARNING: u8 = 2;
}

#[derive(Debug, Default, FromRow, Serialize, Deserialize)]
pub struct Event {
  #[serde(skip_serializing, skip_deserializing)]
  pub id: u64,
  #[serde(skip_serializing, skip_deserializing)]
  pub check_id: u64,
  #[serde(skip_serializing)]
  pub outage_id: Option<u64>,
  pub site: String,
  pub status: u8,
  pub message: String,
  pub created_at: Option<DateTime<Utc>>,
}

impl Event {
  pub async fn for_outage(conn: &mut MySqlConnection, outage: &SiteOutage) -> Result<Vec<Event>> {
    let events = sqlx::query_as::<_, Event>(
      "
        SELECT id, check_id, outage_id, site, status, message, created_at
        FROM events
        WHERE outage_id = ?
      ",
    )
    .bind(outage.id)
    .fetch_all(&mut *conn)
    .await?;

    Ok(events)
  }

  pub async fn insert(&self, conn: &mut MySqlConnection, outage: Option<&SiteOutage>, site: &str) -> Result<()> {
    sqlx::query(
      "
        INSERT INTO events (check_id, outage_id, site, status, message, created_at)
        VALUES ( ?, ?, ?, ?, ?, NOW() )
      ",
    )
    .bind(self.check_id)
    .bind(outage.map(|outage| outage.id))
    .bind(site)
    .bind(self.status)
    .bind(&self.message)
    .execute(conn)
    .await?;

    Ok(())
  }

  pub async fn delete_before(conn: &mut MySqlConnection, epoch: &NaiveDateTime) -> Result<u64> {
    let result = sqlx::query(
      "
        DELETE events FROM events
        LEFT JOIN site_outages AS outages
        ON outages.id = events.outage_id
        WHERE ended_on IS NOT NULL AND ended_on < ?
      ",
    )
    .bind(epoch)
    .execute(conn)
    .await?;

    Ok(result.rows_affected())
  }
}

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use chrono::NaiveDate;
  use uuid::Uuid;

  use super::Event;
  use crate::{model::SiteOutage, spec};

  #[tokio::test]
  async fn for_outage() -> Result<()> {
    let pool = spec::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(None, None, "for_outage()", None).await?;
    pool.create_unresolved_outage(None, None).await?;

    let outage = SiteOutage { id: 1, ..Default::default() };
    let events = Event::for_outage(&mut *conn, &outage).await?;

    assert_eq!(events.len(), 1);
    assert_eq!(&events[0].message, "failure");

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn insert() -> Result<()> {
    let pool = spec::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(None, None, "insert()", None).await?;
    pool.create_unresolved_outage(None, None).await?;

    let outage = SiteOutage { id: 1, ..Default::default() };
    let event = Event {
      check_id: 1,
      status: 1,
      message: "new failure".to_string(),
      ..Default::default()
    };

    event.insert(&mut *conn, Some(&outage), "@controller").await?;

    let event = sqlx::query_as::<_, (u8, String)>(r#"SELECT status, message FROM events ORDER BY id DESC LIMIT 1"#)
      .fetch_one(&*pool)
      .await?;

    assert_eq!(event.0, 1);
    assert_eq!(&event.1, "new failure");

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn delete_before() -> Result<()> {
    let pool = spec::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(None, None, "delete_before()", None).await?;
    pool.create_unresolved_outage(Some(1), Some(Uuid::new_v4().to_string())).await?;
    pool.create_resolved_outage(Some(2), Some(Uuid::new_v4().to_string())).await?;

    Event::delete_before(&mut *conn, &NaiveDate::from_ymd(2021, 2, 1).and_hms(0, 0, 0)).await?;

    let events = sqlx::query_as::<_, (u64,)>(r#"SELECT id FROM events"#).fetch_all(&*pool).await?;

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].0, 1);

    pool.cleanup().await;

    Ok(())
  }
}
