use anyhow::Result;
use chrono::Utc;
use sqlx::{FromRow, MySqlConnection};

use crate::{
  api::{
    error::{server_error, AppError},
    types as api,
  },
  ext,
  handlers::*,
  model::{specs, Alerter, CheckKind, Duration, Event, SiteOutage},
};

#[derive(Debug, Default, FromRow, Serialize, Deserialize)]
pub struct Check {
  #[serde(skip_serializing, skip_deserializing)]
  pub id: u64,
  #[serde(skip_deserializing)]
  pub uuid: String,
  #[serde(skip_serializing, skip_deserializing)]
  pub alerter_id: Option<u64>,
  pub name: String,
  #[serde(default = "ext::to_true")]
  pub enabled: bool,
  #[serde(skip_serializing, skip_deserializing)]
  pub kind: CheckKind,
  pub sites: String,
  pub interval: Duration,
  pub site_threshold: u8,
  pub passing_threshold: u8,
  pub failing_threshold: u8,
  #[serde(default = "ext::to_false")]
  pub silent: bool,
}

impl Check {
  pub async fn all(conn: &mut MySqlConnection) -> Result<Vec<Check>> {
    let checks = sqlx::query_as::<_, Check>(
      "
        SELECT id, uuid, alerter_id, name, enabled, kind, sites, `interval`, site_threshold, passing_threshold, failing_threshold, silent
        FROM checks
      ",
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(server_error)?;

    Ok(checks)
  }

  pub async fn enabled(conn: &mut MySqlConnection) -> Result<Vec<Check>> {
    let checks = sqlx::query_as::<_, Check>(
      "
        SELECT id, uuid, alerter_id, name, enabled, kind, sites, `interval`, site_threshold, passing_threshold, failing_threshold, silent
        FROM checks
        WHERE enabled = 1
      ",
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(server_error)?;

    Ok(checks)
  }

  pub async fn by_id(conn: &mut MySqlConnection, id: u64) -> Result<Check> {
    let check = sqlx::query_as::<_, Check>(
      "
        SELECT id, uuid, alerter_id, name, enabled, kind, sites, `interval`, site_threshold, passing_threshold, failing_threshold, silent
        FROM checks
        WHERE id = ?
      ",
    )
    .bind(id)
    .fetch_one(&mut *conn)
    .await
    .map_err(server_error)?;

    Ok(check)
  }

  pub async fn by_uuid(conn: &mut MySqlConnection, uuid: &str) -> Result<Check> {
    let check = sqlx::query_as::<_, Check>(
      "
        SELECT id, uuid, alerter_id, name, enabled, kind, sites, `interval`, site_threshold, passing_threshold, failing_threshold, silent
        FROM checks
        WHERE uuid = ?
      ",
    )
    .bind(uuid)
    .fetch_one(conn)
    .await
    .map_err(|err| match err {
      sqlx::Error::RowNotFound => AppError::ResourceNotFound(anyhow!(err).context("unknown check")),
      err => server_error(err),
    })?;

    Ok(check)
  }

  pub async fn insert(self, conn: &mut MySqlConnection) -> Result<Check> {
    sqlx::query(
      "
        INSERT INTO checks ( uuid, alerter_id, name, enabled, kind, sites, `interval`, site_threshold, passing_threshold, failing_threshold, silent )
        VALUES ( ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ? )
      ",
    )
    .bind(&self.uuid)
    .bind(self.alerter_id)
    .bind(self.name)
    .bind(self.enabled)
    .bind(self.kind)
    .bind(&self.sites)
    .bind(self.interval)
    .bind(self.site_threshold)
    .bind(self.passing_threshold)
    .bind(self.failing_threshold)
    .bind(self.silent)
    .execute(&mut *conn)
    .await
    .map_err(server_error)?;

    let check = Check::by_uuid(&mut *conn, &self.uuid).await?;

    Ok(check)
  }

  pub async fn update(self, conn: &mut MySqlConnection) -> Result<Check> {
    sqlx::query(
      "
        UPDATE checks
        SET alerter_id = ?, name = ?, enabled = ?, kind = ?, sites = ?, `interval` = ?, site_threshold = ?, passing_threshold = ?, failing_threshold = ?, silent = ?
        WHERE id = ?
      ",
    )
    .bind(self.alerter_id)
    .bind(self.name)
    .bind(self.enabled)
    .bind(self.kind)
    .bind(&self.sites)
    .bind(self.interval)
    .bind(self.site_threshold)
    .bind(self.passing_threshold)
    .bind(self.failing_threshold)
    .bind(self.silent)
    .bind(self.id)
    .execute(&mut *conn)
    .await
    .map_err(server_error)?;

    let check = Check::by_uuid(&mut *conn, &self.uuid).await?;

    Ok(check)
  }

  pub async fn delete(conn: &mut MySqlConnection, uuid: &str) -> Result<()> {
    sqlx::query(
      "
        UPDATE checks
        SET enabled = 0
        WHERE uuid = ?
      ",
    )
    .bind(uuid)
    .execute(conn)
    .await
    .map_err(server_error)?;

    Ok(())
  }

  pub async fn last_event(&self, conn: &mut MySqlConnection, site: &str) -> Result<Option<Event>> {
    let event = sqlx::query_as::<_, Event>(
      "
        SELECT id, check_id, outage_id, site, status, message, created_at
        FROM events
        WHERE check_id = ? AND site = ?
        ORDER BY created_at DESC
        LIMIT 1
      ",
    )
    .bind(self.id)
    .bind(site)
    .fetch_one(&mut *conn)
    .await;

    match event {
      Ok(event) => Ok(Some(event)),

      Err(err) => match err {
        sqlx::Error::RowNotFound => Ok(None),
        err => Err(err.into()),
      },
    }
  }

  pub async fn spec(&self, conn: &mut MySqlConnection) -> Result<api::Spec> {
    use api::Spec;
    use CheckKind::*;

    match self.kind {
      Ping => specs::Ping::for_check(conn, self).await.map(Spec::Ping),
      Dns => specs::Dns::for_check(conn, self).await.map(Spec::Dns),
      Http => specs::Http::for_check(conn, self).await.map(Spec::Http),
      Tcp => specs::Tcp::for_check(conn, self).await.map(Spec::Tcp),
      Udp => specs::Udp::for_check(conn, self).await.map(Spec::Udp),
      Tls => specs::Tls::for_check(conn, self).await.map(Spec::Tls),
      PlayStore => specs::PlayStore::for_check(conn, self).await.map(Spec::PlayStore),
      AppStore => specs::AppStore::for_check(conn, self).await.map(Spec::AppStore),
      Whois => specs::Whois::for_check(conn, self).await.map(Spec::Whois),
    }
  }

  pub fn handler(&self) -> Box<dyn Handler + Sync + '_> {
    use CheckKind::*;

    match self.kind {
      Ping => Box::new(PingHandler { check: &self }),
      Dns => Box::new(DnsHandler { check: &self }),
      Http => Box::new(HttpHandler { check: &self }),
      Tcp => Box::new(TcpHandler { check: &self }),
      Udp => Box::new(UdpHandler { check: &self }),
      Tls => Box::new(TlsHandler { check: &self }),
      PlayStore => Box::new(PlayStoreHandler { check: &self }),
      AppStore => Box::new(AppStoreHandler { check: &self }),
      Whois => Box::new(WhoisHandler { check: &self }),
    }
  }

  pub async fn stale(&self, conn: &mut MySqlConnection, site: &str) -> bool {
    let event = self.last_event(&mut *conn, site).await.unwrap_or(None);

    match event {
      None => true,

      Some(event) => {
        if let Some(date) = event.created_at {
          Utc::now().signed_duration_since(date) >= chrono::Duration::from_std(*self.interval).unwrap()
        } else {
          false
        }
      }
    }
  }

  pub async fn alerter(&self, conn: &mut MySqlConnection) -> Option<Alerter> {
    match self.alerter_id {
      Some(id) => Alerter::by_id(conn, id).await.ok(),
      None => None,
    }
  }

  pub async fn alert(&self, conn: &mut MySqlConnection, outage: &str) {
    if !self.silent {
      let inner = async move || -> Result<()> {
        let outage = SiteOutage::by_uuid(&mut *conn, &outage).await?;

        if let Some(alerter) = self.alerter(&mut *conn).await {
          alerter.webhook().alert(&mut *conn, &self, &outage).await?;
        }

        Ok(())
      };

      if let Err(err) = inner().await {
        crate::log_error(&err);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration as StdDuration;

  use anyhow::Result;
  use uuid::Uuid;

  use crate::spec;

  use super::{Check, CheckKind, Event};
  use crate::model::Duration;

  #[tokio::test]
  async fn list() -> Result<()> {
    let pool = spec::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(None, None, "all()", None).await?;

    let checks = Check::all(&mut *conn).await?;

    assert_eq!(checks.len(), 1);
    assert_eq!(&checks[0].name, "all()");
    assert_eq!(checks[0].kind, CheckKind::Tcp);
    assert_eq!(checks[0].enabled, true);
    assert_eq!(*checks[0].interval, *Duration::from(10));
    assert_eq!(checks[0].passing_threshold, 2);
    assert_eq!(checks[0].failing_threshold, 2);
    assert_eq!(checks[0].silent, false);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn enabled() -> Result<()> {
    let pool = spec::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(Some(1), Some(Uuid::new_v4().to_string()), "enabled()", Some(true)).await?;
    pool.create_check(Some(2), Some(Uuid::new_v4().to_string()), "enabled()", Some(false)).await?;

    let checks = Check::enabled(&mut *conn).await?;

    assert_eq!(checks.len(), 1);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn by_uuid() -> Result<()> {
    let pool = spec::db_client().await?;

    pool.create_check(None, None, "by_uuid()", None).await?;

    let check = sqlx::query_as::<_, (String, String, bool, u64)>(r#"SELECT name, kind, enabled, `interval` FROM checks WHERE uuid = "dd9a531a-1b0b-4a12-bc09-e5637f916261""#)
      .fetch_one(&*pool)
      .await?;

    assert_eq!(&check.0, "by_uuid()");
    assert_eq!(&check.1, "tcp");
    assert_eq!(check.2, true);
    assert_eq!(check.3, 10);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn by_id() -> Result<()> {
    let pool = spec::db_client().await?;

    pool.create_check(None, None, "by_id()", None).await?;

    let check = sqlx::query_as::<_, (String, String, bool, u64)>(r#"SELECT name, kind, enabled, `interval` FROM checks WHERE id = 1"#)
      .fetch_one(&*pool)
      .await?;

    assert_eq!(&check.0, "by_id()");
    assert_eq!(&check.1, "tcp");
    assert_eq!(check.2, true);
    assert_eq!(check.3, 10);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn insert() -> Result<()> {
    let pool = spec::db_client().await?;
    let mut conn = pool.acquire().await?;

    let check = Check {
      id: 1,
      alerter_id: None,
      uuid: "dd9a531a-1b0b-4a12-bc09-e5637f916261".to_string(),
      name: "create()".to_string(),
      kind: CheckKind::Tcp,
      sites: "@controller".to_string(),
      enabled: false,
      interval: Duration::from(10),
      site_threshold: 2,
      passing_threshold: 10,
      failing_threshold: 10,
      silent: false,
    };

    check.insert(&mut *conn).await?;

    let check = sqlx::query_as::<_, (String, bool, u64)>(r#"SELECT name, enabled, `interval` FROM checks WHERE uuid = "dd9a531a-1b0b-4a12-bc09-e5637f916261""#)
      .fetch_one(&*pool)
      .await?;

    assert_eq!(&check.0, "create()");
    assert_eq!(check.1, false);
    assert_eq!(check.2, 10);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn update() -> Result<()> {
    let pool = spec::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(None, None, "update()", None).await?;

    let update = Check {
      id: 1,
      alerter_id: None,
      uuid: "dd9a531a-1b0b-4a12-bc09-e5637f916261".to_string(),
      name: "new_update()".to_string(),
      kind: CheckKind::Tcp,
      sites: "@controller".to_string(),
      enabled: false,
      interval: Duration::from(10),
      site_threshold: 2,
      passing_threshold: 10,
      failing_threshold: 10,
      silent: false,
    };

    update.update(&mut *conn).await?;

    let check = sqlx::query_as::<_, (String, bool, u64)>(r#"SELECT name, enabled, `interval` FROM checks WHERE uuid = "dd9a531a-1b0b-4a12-bc09-e5637f916261""#)
      .fetch_one(&*pool)
      .await?;

    assert_eq!(&check.0, "new_update()");
    assert_eq!(check.1, false);
    assert_eq!(check.2, 10);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn delete() -> Result<()> {
    let pool = spec::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(None, None, "delete()", None).await?;

    let check = Check {
      uuid: "dd9a531a-1b0b-4a12-bc09-e5637f916261".to_string(),
      ..Default::default()
    };

    Check::delete(&mut *conn, &check.uuid).await?;

    let deleted = sqlx::query_as::<_, (bool,)>(r#"SELECT enabled FROM checks WHERE id = 1"#).fetch_one(&*pool).await?;
    assert_eq!(deleted.0, false);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn last_event() -> Result<()> {
    let pool = spec::db_client().await?;
    let mut conn = pool.acquire().await?;

    let check = Check {
      uuid: Uuid::new_v4().to_string(),
      name: "Test check".to_string(),
      kind: CheckKind::Tcp,
      enabled: true,
      interval: Duration::from(5),
      passing_threshold: 1,
      failing_threshold: 1,
      silent: true,
      ..Default::default()
    };

    let check = check.insert(&mut *conn).await?;

    let mut event = Event {
      check_id: check.id,
      status: 0,
      message: "First event".to_string(),
      ..Default::default()
    };

    event.insert(&mut *conn, None, "@controller").await?;

    tokio::time::delay_for(StdDuration::from_secs(2)).await;

    event.message = "Last event".to_string();
    event.insert(&mut *conn, None, "@controller").await?;

    let last = check.last_event(&mut *conn, "@controller").await?;

    assert_eq!(last.is_some(), true);
    assert_eq!(&last.unwrap().message, "Last event");

    pool.cleanup().await;

    Ok(())
  }
}
