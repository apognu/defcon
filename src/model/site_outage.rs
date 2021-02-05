use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use kvlogger::*;
use sqlx::{Done, FromRow, MySqlConnection};
use uuid::Uuid;

use crate::{
  api::error::Shortable,
  model::{Check, Event},
};

#[derive(Debug)]
enum OutageRef {
  New,
  Existing(SiteOutage),
}

#[derive(Debug, Default, FromRow, Serialize, Deserialize)]
pub struct SiteOutage {
  #[serde(skip_serializing, skip_deserializing)]
  pub id: u64,
  pub uuid: String,
  #[serde(skip_serializing, skip_deserializing)]
  pub check_id: u64,
  pub site: String,
  pub passing_strikes: u8,
  pub failing_strikes: u8,
  pub started_on: Option<DateTime<Utc>>,
  pub ended_on: Option<DateTime<Utc>>,
}

impl SiteOutage {
  pub async fn between(conn: &mut MySqlConnection, from: NaiveDateTime, end: NaiveDateTime) -> Result<Vec<SiteOutage>> {
    let outages = sqlx::query_as::<_, SiteOutage>(
      "
        SELECT outages.id, outages.uuid, outages.check_id, outages.site, outages.passing_strikes, outages.failing_strikes, outages.started_on, outages.ended_on
        FROM site_outages AS outages
        INNER JOIN checks
        ON checks.id = outages.check_id
        WHERE
          checks.enabled = 1 AND outages.failing_strikes >= checks.failing_threshold AND
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

  pub async fn current(conn: &mut MySqlConnection) -> Result<Vec<SiteOutage>> {
    let outages = sqlx::query_as::<_, SiteOutage>(
      "
        SELECT outages.id, outages.uuid, outages.check_id, outages.site, outages.passing_strikes, outages.failing_strikes, outages.started_on, outages.ended_on
        FROM site_outages AS outages
        INNER JOIN checks
        ON checks.id = outages.check_id
        WHERE outages.ended_on IS NULL AND checks.enabled = 1 AND outages.failing_strikes >= checks.failing_threshold
      ",
    )
    .fetch_all(&mut *conn)
    .await
    .short()?;

    Ok(outages)
  }

  pub async fn by_uuid(conn: &mut MySqlConnection, uuid: &str) -> Result<SiteOutage> {
    let outage = sqlx::query_as::<_, SiteOutage>(
      "
        SELECT id, uuid, check_id, site, passing_strikes, failing_strikes, started_on, ended_on
        FROM site_outages
        WHERE uuid = ?
      ",
    )
    .bind(uuid)
    .fetch_one(&mut *conn)
    .await
    .short()?;

    Ok(outage)
  }

  async fn for_check(conn: &mut MySqlConnection, check: &Check, site: &str) -> Result<OutageRef> {
    let outage = sqlx::query_as::<_, SiteOutage>(
      "
        SELECT id, uuid, check_id, site, passing_strikes, failing_strikes, started_on, ended_on
        FROM site_outages
        WHERE check_id = ? AND site = ? AND ended_on IS NULL
      ",
    )
    .bind(check.id)
    .bind(site)
    .fetch_one(&mut *conn)
    .await;

    match outage {
      Ok(outage) => Ok(OutageRef::Existing(outage)),

      Err(err) => match err {
        sqlx::Error::RowNotFound => Ok(OutageRef::New),
        err => Err(err).short()?,
      },
    }
  }

  pub async fn insert(conn: &mut MySqlConnection, check: &Check, event: &Event) -> Result<Option<SiteOutage>> {
    let outage = SiteOutage::for_check(conn, check, &event.site).await;

    let outage = match outage {
      Ok(OutageRef::Existing(outage)) => {
        if outage.failing_strikes < check.failing_threshold && event.status == 1 {
          sqlx::query(
            "
              UPDATE site_outages
              SET failing_strikes = failing_strikes + 1, passing_strikes = 0
              WHERE id = ?
            ",
          )
          .bind(outage.id)
          .execute(&mut *conn)
          .await
          .short()?;

          if outage.failing_strikes + 1 == check.failing_threshold {
            kvlog!(Info, "site outage started", {
              "site" => event.site,
              "kind" => check.kind,
              "check" => check.uuid,
              "failed" => format!("{}/{}", outage.failing_strikes + 1, check.failing_threshold),
              "passed" => format!("0/{}", check.passing_threshold)
            });
          }
        }

        if outage.passing_strikes < check.passing_threshold && event.status == 0 {
          let ended_on = if outage.passing_strikes + 1 == check.passing_threshold {
            kvlog!(Info, "site outage resolved", {
              "site" => event.site,
              "kind" => check.kind,
              "check" => check.uuid,
              "failed" => format!("{}/{}", outage.failing_strikes, check.failing_threshold),
              "passed" => format!("{}/{}", outage.passing_strikes + 1, check.passing_threshold)
            });

            Some(Utc::now())
          } else {
            None
          };

          sqlx::query(
            "
              UPDATE site_outages
              SET passing_strikes = passing_strikes + 1, ended_on = ?
              WHERE id = ?
            ",
          )
          .bind(ended_on)
          .bind(outage.id)
          .execute(&mut *conn)
          .await
          .short()?;
        }

        let outage = SiteOutage::by_uuid(&mut *conn, &outage.uuid).await?;

        Some(outage)
      }

      Ok(OutageRef::New) => {
        if event.status != 0 {
          let uuid = Uuid::new_v4().to_string();

          sqlx::query(
            "
              INSERT INTO site_outages (uuid, check_id, site, passing_strikes, failing_strikes, started_on)
              VALUES ( ?, ?, ?, 0, 1, NOW() )
            ",
          )
          .bind(&uuid)
          .bind(event.check_id)
          .bind(&event.site)
          .execute(&mut *conn)
          .await
          .short()?;

          let outage = SiteOutage::by_uuid(&mut *conn, &uuid).await?;

          if check.failing_threshold == 1 {
            kvlog!(Info, "site outage started", {
              "site" => event.site,
              "kind" => check.kind,
              "check" => check.uuid,
              "failed" => format!("1/{}", check.failing_threshold),
              "passed" => format!("0/{}", check.passing_threshold)
            });
          }

          Some(outage)
        } else {
          None
        }
      }

      Err(err) => {
        log::error!("{:#}", err);

        None
      }
    };

    Ok(outage)
  }

  pub async fn delete_before(conn: &mut MySqlConnection, epoch: &NaiveDateTime) -> Result<u64> {
    let result = sqlx::query(
      "
        DELETE FROM site_outages
        WHERE ended_on IS NOT NULL AND ended_on < ?
      ",
    )
    .bind(epoch)
    .execute(conn)
    .await
    .short()?;

    Ok(result.rows_affected())
  }

  pub async fn count(conn: &mut MySqlConnection, check: &Check) -> Result<i64> {
    let count = sqlx::query_as::<_, (i64,)>(
      "
        SELECT COUNT(site_outages.id)
        FROM site_outages
        INNER JOIN checks
        ON checks.id = site_outages.check_id
        WHERE
          checks.id = ? AND
          site_outages.failing_strikes >= checks.failing_threshold AND
          site_outages.passing_strikes < checks.passing_threshold
      ",
    )
    .bind(check.id)
    .fetch_one(&mut *conn)
    .await
    .short()?;

    Ok(count.0)
  }
}

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use chrono::NaiveDate;
  use uuid::Uuid;

  use crate::{
    model::{Check, Event, SiteOutage},
    tests,
  };

  use super::OutageRef;

  #[tokio::test]
  async fn between() -> Result<()> {
    let pool = tests::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(None, None, "between()", None, None).await?;
    pool.create_unresolved_site_outage(Some(1), Some(Uuid::new_v4().to_string())).await?;
    pool.create_resolved_site_outage(Some(2), Some(Uuid::new_v4().to_string())).await?;

    let start = NaiveDate::from_ymd(2021, 1, 1).and_hms(0, 0, 0);
    let end = NaiveDate::from_ymd(2021, 2, 1).and_hms(0, 0, 0);
    let outages = SiteOutage::between(&mut *conn, start, end).await?;

    assert_eq!(outages.len(), 2);

    let start = NaiveDate::from_ymd(2021, 1, 1).and_hms(0, 0, 0);
    let end = NaiveDate::from_ymd(2021, 1, 5).and_hms(0, 0, 0);
    let outages = SiteOutage::between(&mut *conn, start, end).await?;

    assert_eq!(outages.len(), 1);

    let start = NaiveDate::from_ymd(2020, 12, 1).and_hms(0, 0, 0);
    let end = NaiveDate::from_ymd(2020, 12, 2).and_hms(0, 0, 0);
    let outages = SiteOutage::between(&mut *conn, start, end).await?;

    assert_eq!(outages.len(), 0);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn by_uuid() -> Result<()> {
    let pool = tests::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(None, None, "by_uuid()", None, None).await?;
    pool.create_unresolved_site_outage(None, None).await?;

    let outage = SiteOutage::by_uuid(&mut *conn, "dd9a531a-1b0b-4a12-bc09-e5637f916261").await?;

    assert_eq!(outage.id, 1);
    assert_eq!(outage.uuid, "dd9a531a-1b0b-4a12-bc09-e5637f916261".to_string());

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn for_check() -> Result<()> {
    let pool = tests::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(None, None, "by_uuid()", None, None).await?;

    let check = Check { id: 1, ..Default::default() };

    let outage = SiteOutage::for_check(&mut *conn, &check, "@controller").await?;
    assert!(matches!(outage, OutageRef::New));

    pool.create_unresolved_site_outage(None, None).await?;

    let outage = SiteOutage::for_check(&mut *conn, &check, "@controller").await?;
    assert!(matches!(outage, OutageRef::Existing(SiteOutage { id: 1, ref uuid, .. }) if uuid == "dd9a531a-1b0b-4a12-bc09-e5637f916261" ));

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn insert() -> Result<()> {
    let pool = tests::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(None, None, "insert()", None, None).await?;

    let check = Check {
      id: 1,
      failing_threshold: 2,
      passing_threshold: 2,
      ..Default::default()
    };

    let event = Event {
      check_id: 1,
      status: 1,
      message: "failure".to_string(),
      ..Default::default()
    };

    SiteOutage::insert(&mut *conn, &check, &event).await?;
    let outage = sqlx::query_as::<_, (u8,)>("SELECT failing_strikes FROM site_outages WHERE id = 1").fetch_one(&*pool).await?;
    assert_eq!(outage, (1,));

    SiteOutage::insert(&mut *conn, &check, &event).await?;
    let outage = sqlx::query_as::<_, (u8,)>("SELECT failing_strikes FROM site_outages WHERE id = 1").fetch_one(&*pool).await?;
    assert_eq!(outage, (2,));

    SiteOutage::insert(&mut *conn, &check, &event).await?;
    let outage = sqlx::query_as::<_, (u8,)>("SELECT failing_strikes FROM site_outages WHERE id = 1").fetch_one(&*pool).await?;
    assert_eq!(outage, (2,));

    let event = Event {
      check_id: 1,
      status: 0,
      message: "success".to_string(),
      ..Default::default()
    };

    SiteOutage::insert(&mut *conn, &check, &event).await?;

    let outage = sqlx::query_as::<_, (u8, u8)>("SELECT passing_strikes, failing_strikes FROM site_outages WHERE id = 1")
      .fetch_one(&*pool)
      .await?;

    assert_eq!(outage, (1, 2));

    SiteOutage::insert(&mut *conn, &check, &event).await?;

    let outage = sqlx::query_as::<_, (u8, u8)>("SELECT passing_strikes, failing_strikes FROM site_outages WHERE id = 1")
      .fetch_one(&*pool)
      .await?;

    assert_eq!(outage, (2, 2));

    Ok(())
  }

  #[tokio::test]
  async fn current() -> Result<()> {
    let pool = tests::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(None, None, "between()", None, None).await?;
    pool.create_unresolved_site_outage(Some(1), Some(Uuid::new_v4().to_string())).await?;
    pool.create_resolved_site_outage(Some(2), Some(Uuid::new_v4().to_string())).await?;

    let outages = SiteOutage::current(&mut *conn).await?;

    assert_eq!(outages.len(), 1);
    assert_eq!(outages[0].id, 1);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn delete_before() -> Result<()> {
    let pool = tests::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(None, None, "delete_before()", None, None).await?;
    pool.create_unresolved_site_outage(Some(1), Some(Uuid::new_v4().to_string())).await?;
    pool.create_resolved_site_outage(Some(2), Some(Uuid::new_v4().to_string())).await?;

    let epoch = NaiveDate::from_ymd(2021, 2, 1).and_hms(0, 0, 0);
    Event::delete_before(&mut *conn, &epoch).await?;
    SiteOutage::delete_before(&mut *conn, &epoch).await?;

    let events = sqlx::query_as::<_, (u64,)>(r#"SELECT id FROM events"#).fetch_all(&*pool).await?;

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].0, 1);

    pool.cleanup().await;

    Ok(())
  }
}
