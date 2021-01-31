use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use kvlogger::*;
use sqlx::{Done, FromRow, MySqlConnection};
use uuid::Uuid;

use crate::{
  api::error::{server_error, AppError},
  model::{Check, Event},
};

enum OutageRef {
  New,
  Existing(Outage),
}

#[derive(Debug, Default, FromRow, Serialize)]
pub struct Outage {
  #[serde(skip_serializing)]
  pub id: u64,
  pub uuid: String,
  #[serde(skip_serializing)]
  pub check_id: u64,
  pub passing_strikes: u8,
  pub failing_strikes: u8,
  pub started_on: Option<DateTime<Utc>>,
  pub ended_on: Option<DateTime<Utc>>,
  pub comment: Option<String>,
}

impl Outage {
  pub async fn between(conn: &mut MySqlConnection, from: NaiveDateTime, end: NaiveDateTime) -> Result<Vec<Outage>> {
    let outages = sqlx::query_as::<_, Outage>(
      "
        SELECT outages.id, outages.uuid, outages.check_id, outages.passing_strikes, outages.failing_strikes, outages.started_on, outages.ended_on, outages.comment
        FROM outages
        INNER JOIN checks
        ON checks.id = outages.check_id
        WHERE
          checks.enabled = 1 AND outages.failing_strikes >= checks.failing_threshold AND
          outages.started_on > ? AND (outages.ended_on IS NULL OR outages.ended_on < ?)
      ",
    )
    .bind(from)
    .bind(end)
    .fetch_all(&mut *conn)
    .await
    .map_err(server_error)?;

    Ok(outages)
  }

  pub async fn current(conn: &mut MySqlConnection) -> Result<Vec<Outage>> {
    let outages = sqlx::query_as::<_, Outage>(
      "
        SELECT outages.id, outages.uuid, outages.check_id, outages.passing_strikes, outages.failing_strikes, outages.started_on, outages.ended_on, outages.comment
        FROM outages
        INNER JOIN checks
        ON checks.id = outages.check_id
        WHERE outages.ended_on IS NULL AND checks.enabled = 1 AND outages.failing_strikes >= checks.failing_threshold
      ",
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(server_error)?;

    Ok(outages)
  }

  pub async fn by_uuid(conn: &mut MySqlConnection, uuid: &str) -> Result<Outage> {
    let outage = sqlx::query_as::<_, Outage>(
      "
        SELECT id, uuid, check_id, passing_strikes, failing_strikes, started_on, ended_on, comment
        FROM outages
        WHERE uuid = ?
      ",
    )
    .bind(uuid)
    .fetch_one(&mut *conn)
    .await
    .map_err(|err| match err {
      sqlx::Error::RowNotFound => AppError::ResourceNotFound(anyhow!(err).context("unknown check UUID")),
      err => server_error(err),
    })?;

    Ok(outage)
  }

  async fn for_check(conn: &mut MySqlConnection, check_id: u64) -> Result<OutageRef> {
    let outage = sqlx::query_as::<_, Outage>(
      "
        SELECT id, uuid, check_id, passing_strikes, failing_strikes, started_on, ended_on, comment
        FROM outages
        WHERE check_id = ? AND ended_on IS NULL
      ",
    )
    .bind(check_id)
    .fetch_one(&mut *conn)
    .await;

    match outage {
      Ok(outage) => Ok(OutageRef::Existing(outage)),

      Err(err) => match err {
        sqlx::Error::RowNotFound => Ok(OutageRef::New),
        err => Err(err.into()),
      },
    }
  }

  pub async fn insert(conn: &mut MySqlConnection, check: &Check, event: &Event) -> Result<Option<Outage>> {
    let outage = Outage::for_check(conn, event.check_id).await;

    let outage = match outage {
      Ok(OutageRef::Existing(outage)) => {
        if outage.failing_strikes < check.failing_threshold && event.status == 1 {
          sqlx::query(
            "
              UPDATE outages
              SET failing_strikes = failing_strikes + 1, passing_strikes = 0
              WHERE id = ?
            ",
          )
          .bind(outage.id)
          .execute(&mut *conn)
          .await?;

          if outage.failing_strikes + 1 == check.failing_threshold {
            kvlog!(Info, "outage started", {
              "kind" => check.kind,
              "check" => check.uuid,
              "failed" => format!("{}/{}", outage.failing_strikes + 1, check.failing_threshold),
              "passed" => format!("0/{}", check.passing_threshold)
            });

            check.alert(&mut *conn, &outage.uuid).await;
          }
        }

        if outage.passing_strikes < check.passing_threshold && event.status == 0 {
          let (ended_on, alert) = if outage.passing_strikes + 1 == check.passing_threshold {
            kvlog!(Info, "outage resolved", {
              "kind" => check.kind,
              "check" => check.uuid,
              "failed" => format!("{}/{}", outage.failing_strikes, check.failing_threshold),
              "passed" => format!("{}/{}", outage.passing_strikes + 1, check.passing_threshold)
            });

            (Some(Utc::now()), true)
          } else {
            (None, false)
          };

          sqlx::query(
            "
            UPDATE outages
            SET passing_strikes = passing_strikes + 1, ended_on = ?
            WHERE id = ?
          ",
          )
          .bind(ended_on)
          .bind(outage.id)
          .execute(&mut *conn)
          .await?;

          if alert {
            check.alert(&mut *conn, &outage.uuid).await;
          }
        }

        Some(outage)
      }

      Ok(OutageRef::New) => {
        if event.status != 0 {
          let uuid = Uuid::new_v4().to_string();

          sqlx::query(
            "
              INSERT INTO outages (uuid, check_id, passing_strikes, failing_strikes, started_on)
              VALUES ( ?, ?, 0, 1, NOW() )
            ",
          )
          .bind(&uuid)
          .bind(event.check_id)
          .execute(&mut *conn)
          .await?;

          let outage = Outage::by_uuid(&mut *conn, &uuid).await?;

          if check.failing_threshold == 1 {
            kvlog!(Info, "outage started", {
              "kind" => check.kind,
              "check" => check.uuid,
              "failed" => format!("1/{}", check.failing_threshold),
              "passed" => format!("0/{}", check.passing_threshold)
            });

            check.alert(&mut *conn, &outage.uuid).await;
          }

          Some(outage)
        } else {
          None
        }
      }

      Err(err) => {
        crate::log_error(&err);

        None
      }
    };

    Ok(outage)
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
    .map_err(|err| match err {
      sqlx::Error::RowNotFound => AppError::ResourceNotFound(anyhow!(err).context("unknown check UUID")),
      err => server_error(err),
    })?;

    Ok(())
  }

  pub async fn delete_before(pool: &mut MySqlConnection, epoch: &NaiveDateTime) -> Result<u64> {
    let result = sqlx::query(
      "
        DELETE FROM outages
        WHERE ended_on IS NOT NULL AND ended_on < ?
      ",
    )
    .bind(epoch)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
  }
}
