use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, MySqlConnection};
use uuid::Uuid;

use crate::model::Check;

#[derive(Debug, FromRow, Default, Serialize)]
pub struct Outage {
  #[serde(skip_serializing, skip_deserializing)]
  pub id: u64,
  #[serde(skip_serializing, skip_deserializing)]
  pub check_id: u64,
  pub uuid: String,
  pub started_on: Option<DateTime<Utc>>,
  pub ended_on: Option<DateTime<Utc>>,
  pub comment: Option<String>,
}

impl Outage {
  pub async fn by_uuid(conn: &mut MySqlConnection, uuid: &str) -> Result<Outage> {
    let outage = sqlx::query_as::<_, Outage>(
      "
        SELECT id, check_id, uuid, started_on, ended_on, comment
        FROM outages
        WHERE uuid = ?
      ",
    )
    .bind(uuid)
    .fetch_one(&mut *conn)
    .await?;

    Ok(outage)
  }

  pub async fn insert(conn: &mut MySqlConnection, check: &Check) -> Result<Outage> {
    let uuid = Uuid::new_v4().to_string();

    sqlx::query(
      "
        INSERT INTO outages (check_id, uuid, started_on)
        VALUES ( ?, ?, NOW() )
      ",
    )
    .bind(check.id)
    .bind(&uuid)
    .execute(&mut *conn)
    .await?;

    let outage = Outage::by_uuid(conn, &uuid).await?;

    check.alert(&mut *conn, &outage.uuid).await;

    Ok(outage)
  }

  pub async fn resolve(conn: &mut MySqlConnection, check: &Check) -> Result<()> {
    sqlx::query(
      "
        UPDATE outages
        SET ended_on = NOW()
        WHERE check_id = ?
      ",
    )
    .bind(check.id)
    .execute(&mut *conn)
    .await?;

    Ok(())
  }

  pub async fn comment(&self, conn: &mut MySqlConnection, comment: &str) -> Result<()> {
    sqlx::query(
      "
        UPDATE outages
        SET comment = ?
        WHERE uuid = ?
      ",
    )
    .bind(comment)
    .bind(&self.uuid)
    .execute(conn)
    .await?;

    Ok(())
  }
}
