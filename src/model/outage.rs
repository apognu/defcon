use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use kvlogger::*;
use sqlx::{FromRow, MySqlConnection};
use uuid::Uuid;

use crate::{api::error::Shortable, model::Check};

#[derive(Debug, FromRow, Default, Serialize)]
pub struct Outage {
  #[serde(skip)]
  pub id: u64,
  #[serde(skip)]
  pub check_id: u64,
  pub uuid: String,
  pub started_on: Option<DateTime<Utc>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ended_on: Option<DateTime<Utc>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub comment: Option<String>,
}

impl Outage {
  pub async fn between(conn: &mut MySqlConnection, from: NaiveDateTime, end: NaiveDateTime) -> Result<Vec<Outage>> {
    let outages = sqlx::query_as::<_, Outage>(
      "
        SELECT outages.id, check_id, outages.uuid, started_on, ended_on, comment
        FROM outages
        INNER JOIN checks
        ON checks.id = outages.check_id
        WHERE
          checks.enabled = 1 AND
          (
            (outages.started_on <= ? AND outages.ended_on IS NULL) OR
            (outages.started_on <= ? AND outages.ended_on >= ?) OR
            (outages.started_on BETWEEN ? AND ? AND (outages.ended_on IS NULL OR outages.ended_on <= ?))
          )
      ",
    )
    .bind(end)
    .bind(from)
    .bind(from)
    .bind(from)
    .bind(end)
    .bind(end)
    .fetch_all(&mut *conn)
    .await
    .short()?;

    Ok(outages)
  }

  pub async fn current(conn: &mut MySqlConnection) -> Result<Vec<Outage>> {
    let outages = sqlx::query_as::<_, Outage>(
      "
        SELECT outages.id, check_id, outages.uuid, started_on, ended_on, comment
        FROM outages
        INNER JOIN checks
        ON checks.id = outages.check_id
        WHERE outages.ended_on IS NULL AND checks.enabled = 1
      ",
    )
    .fetch_all(&mut *conn)
    .await
    .short()?;

    Ok(outages)
  }

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
    .await
    .short()?;

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
    .await
    .short()?;

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

        kvlog!(Info, "outage confirmed", {
          "check" => check.uuid,
          "outage" => outage.uuid,
          "since" => outage.started_on.map(|dt| dt.to_string()).unwrap_or_else(|| "-".to_string())
        });

        check.alert(&mut *conn, &outage.uuid).await;

        Ok(outage)
      }

      Ok(outage) => Ok(outage),
    }
  }

  pub async fn resolve(conn: &mut MySqlConnection, check: &Check) -> Result<()> {
    if let Ok(outage) = Outage::for_check(conn, check).await {
      kvlog!(Info, "outage resolved", {
        "check" => check.uuid,
        "outage" => outage.uuid,
        "since" => outage.started_on.map(|dt| dt.to_string()).unwrap_or_else(|| "-".to_string())
      });

      let result = sqlx::query(
        "
        UPDATE outages
        SET ended_on = NOW()
        WHERE id = ? AND ended_on IS NULL
      ",
      )
      .bind(outage.id)
      .execute(&mut *conn)
      .await
      .short()?;

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
    .await
    .short()?;

    Ok(())
  }

  pub async fn delete_before(conn: &mut MySqlConnection, epoch: &NaiveDateTime) -> Result<u64> {
    let result = sqlx::query(
      "
        DELETE FROM outages
        WHERE ended_on IS NOT NULL AND ended_on < ?
      ",
    )
    .bind(epoch)
    .execute(conn)
    .await
    .short()?;

    Ok(result.rows_affected())
  }
}
