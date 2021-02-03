use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Done, FromRow, MySqlConnection};
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

  pub async fn for_check(conn: &mut MySqlConnection, check: &Check) -> Result<Outage> {
    let outage = sqlx::query_as::<_, Outage>(
      "
        SELECT id, check_id, uuid, started_on, ended_on, comment
        FROM outages
        WHERE check_id = ? AND ended_on IS NULL
      ",
    )
    .bind(check.id)
    .fetch_one(&mut *conn)
    .await?;

    Ok(outage)
  }

  pub async fn confirm(conn: &mut MySqlConnection, check: &Check) -> Result<Outage> {
    match Outage::for_check(conn, check).await {
      Err(_) => {
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

      Ok(outage) => Ok(outage),
    }
  }

  pub async fn resolve(conn: &mut MySqlConnection, check: &Check) -> Result<()> {
    if let Ok(outage) = Outage::for_check(conn, check).await {
      let result = sqlx::query(
        "
        UPDATE outages
        SET ended_on = NOW()
        WHERE id = ? AND ended_on IS NULL
      ",
      )
      .bind(outage.id)
      .execute(&mut *conn)
      .await?;

      if result.rows_affected() > 0 {
        check.alert(&mut *conn, &outage.uuid).await;
      }
    }

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
