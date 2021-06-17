use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, MySqlConnection};

use crate::{
  api::{error::Shortable, types as api},
  ext,
  handlers::*,
  model::{specs, Alerter, CheckKind, DeadManSwitchLog, Duration, Event, Group, Outage, Site},
  stash::Stash,
};

#[derive(Debug, Default, FromRow, Serialize, Deserialize)]
pub struct Check {
  #[serde(skip)]
  pub id: u64,
  #[serde(skip_deserializing)]
  pub uuid: String,
  #[serde(skip)]
  pub group_id: Option<u64>,
  #[serde(skip)]
  pub alerter_id: Option<u64>,
  pub name: String,
  #[serde(default = "ext::to_true")]
  pub enabled: bool,
  #[serde(skip)]
  pub kind: CheckKind,
  pub interval: Duration,
  #[serde(default = "default_site_threshold")]
  pub site_threshold: u8,
  pub passing_threshold: u8,
  pub failing_threshold: u8,
  #[serde(default = "ext::to_false")]
  pub silent: bool,
}

const fn default_site_threshold() -> u8 {
  1
}

impl Check {
  pub async fn all(conn: &mut MySqlConnection) -> Result<Vec<Check>> {
    let checks = sqlx::query_as::<_, Check>(
      "
        SELECT id, uuid, group_id, alerter_id, name, enabled, kind, `interval`, site_threshold, passing_threshold, failing_threshold, silent
        FROM checks
      ",
    )
    .fetch_all(&mut *conn)
    .await
    .short()?;

    Ok(checks)
  }

  pub async fn enabled(conn: &mut MySqlConnection) -> Result<Vec<Check>> {
    let checks = sqlx::query_as::<_, Check>(
      "
        SELECT id, uuid, group_id, alerter_id, name, enabled, kind, `interval`, site_threshold, passing_threshold, failing_threshold, silent
        FROM checks
        WHERE enabled = 1
      ",
    )
    .fetch_all(&mut *conn)
    .await
    .short()?;

    Ok(checks)
  }

  pub async fn by_id(conn: &mut MySqlConnection, id: u64) -> Result<Check> {
    let check = sqlx::query_as::<_, Check>(
      "
        SELECT id, uuid, group_id, alerter_id, name, enabled, kind, `interval`, site_threshold, passing_threshold, failing_threshold, silent
        FROM checks
        WHERE id = ?
      ",
    )
    .bind(id)
    .fetch_one(&mut *conn)
    .await
    .short()?;

    Ok(check)
  }

  pub async fn by_ids(conn: &mut MySqlConnection, ids: &[u64]) -> Result<Vec<Check>> {
    if ids.is_empty() {
      return Ok(vec![]);
    }

    let ids = ids.iter().map(ToString::to_string).collect::<Vec<String>>().join(",");

    let checks = sqlx::query_as::<_, Check>(&format!(
      "
        SELECT id, uuid, group_id, alerter_id, name, enabled, kind, `interval`, site_threshold, passing_threshold, failing_threshold, silent
        FROM checks
        WHERE id IN ( {} )
      ",
      ids
    ))
    .fetch_all(&mut *conn)
    .await
    .short()?;

    Ok(checks)
  }

  pub async fn by_uuid(conn: &mut MySqlConnection, uuid: &str) -> Result<Check> {
    let check = sqlx::query_as::<_, Check>(
      "
        SELECT id, uuid, group_id, alerter_id, name, enabled, kind, `interval`, site_threshold, passing_threshold, failing_threshold, silent
        FROM checks
        WHERE uuid = ?
      ",
    )
    .bind(uuid)
    .fetch_one(conn)
    .await
    .short()?;

    Ok(check)
  }

  pub async fn insert(self, conn: &mut MySqlConnection) -> Result<Check> {
    {
      sqlx::query(
        "
        INSERT INTO checks ( uuid, group_id, alerter_id, name, enabled, kind, `interval`, site_threshold, passing_threshold, failing_threshold, silent )
        VALUES ( ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ? )
      ",
      )
      .bind(&self.uuid)
      .bind(self.group_id)
      .bind(self.alerter_id)
      .bind(self.name)
      .bind(self.enabled)
      .bind(self.kind)
      .bind(self.interval)
      .bind(self.site_threshold)
      .bind(self.passing_threshold)
      .bind(self.failing_threshold)
      .bind(self.silent)
      .execute(&mut *conn)
      .await
      .short()?;
    }

    let check = Check::by_uuid(&mut *conn, &self.uuid).await?;

    Ok(check)
  }

  pub async fn update(self, conn: &mut MySqlConnection) -> Result<Check> {
    sqlx::query(
      "
        UPDATE checks
        SET group_id = ?, alerter_id = ?, name = ?, enabled = ?, kind = ?, `interval` = ?, site_threshold = ?, passing_threshold = ?, failing_threshold = ?, silent = ?
        WHERE id = ?
      ",
    )
    .bind(self.group_id)
    .bind(self.alerter_id)
    .bind(self.name)
    .bind(self.enabled)
    .bind(self.kind)
    .bind(self.interval)
    .bind(self.site_threshold)
    .bind(self.passing_threshold)
    .bind(self.failing_threshold)
    .bind(self.silent)
    .bind(self.id)
    .execute(&mut *conn)
    .await
    .short()?;

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
    .short()?;

    Ok(())
  }

  pub async fn last_event(&self, conn: &mut MySqlConnection) -> Result<Option<Event>> {
    let event = sqlx::query_as::<_, Event>(
      "
        SELECT id, check_id, outage_id, site, status, message, created_at
        FROM events
        WHERE check_id = ?
        ORDER BY id DESC
        LIMIT 1
      ",
    )
    .bind(self.id)
    .fetch_one(&mut *conn)
    .await
    .map(Some)
    .short()?;

    Ok(event)
  }

  pub async fn last_event_for_site(&self, conn: &mut MySqlConnection, site: &str) -> Result<Option<Event>> {
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
    .await
    .map(Some)
    .short()?;

    Ok(event)
  }

  pub async fn stale(conn: &mut MySqlConnection, site: &str) -> Result<Vec<Check>> {
    let checks = sqlx::query_as::<_, (u64, u64, Option<DateTime<Utc>>)>(
      "
        SELECT
          MAX(checks.id) AS id,
          MAX(checks.interval) AS `interval`,
          MAX(events.created_at) AS event_date
        FROM checks
        INNER JOIN check_sites
        ON check_sites.check_id = checks.id
        LEFT JOIN events
        ON events.check_id = checks.id AND events.site = check_sites.slug
        WHERE checks.enabled = 1 AND check_sites.slug = ?
        GROUP BY checks.id, check_sites.slug
        HAVING event_date IS NULL OR event_date < TIMESTAMPADD(SECOND, -`interval`, NOW());
      ",
    )
    .bind(site)
    .fetch_all(&mut *conn)
    .await
    .short()?;

    let ids: Vec<u64> = checks.iter().map(|check| check.0).collect();
    let checks = Check::by_ids(conn, &ids).await?;

    Ok(checks)
  }

  pub async fn spec(&self, conn: &mut MySqlConnection) -> Result<api::Spec> {
    use api::Spec;
    use CheckKind::*;

    match self.kind {
      #[cfg(feature = "ping")]
      Ping => specs::Ping::for_check(conn, self).await.map(Spec::Ping),
      Dns => specs::Dns::for_check(conn, self).await.map(Spec::Dns),
      Http => specs::Http::for_check(conn, self).await.map(Spec::Http),
      Tcp => specs::Tcp::for_check(conn, self).await.map(Spec::Tcp),
      Udp => specs::Udp::for_check(conn, self).await.map(Spec::Udp),
      Tls => specs::Tls::for_check(conn, self).await.map(Spec::Tls),
      PlayStore => specs::PlayStore::for_check(conn, self).await.map(Spec::PlayStore),
      AppStore => specs::AppStore::for_check(conn, self).await.map(Spec::AppStore),
      Whois => specs::Whois::for_check(conn, self).await.map(Spec::Whois),
      DeadManSwitch => specs::DeadManSwitch::for_check(conn, self).await.map(Spec::DeadManSwitch),
      Unsupported => Ok(Spec::Unsupported),
    }
  }

  pub async fn run(&self, conn: &mut MySqlConnection, config: Arc<Config>, site: &str, stash: Stash) -> Result<Event> {
    use CheckKind::*;

    match self.kind {
      #[cfg(feature = "ping")]
      Ping => PingHandler { check: &self }.check(conn, config, site, stash).await,

      Dns => {
        DnsHandler {
          check: &self,
          resolver: config.checks.dns_resolver,
        }
        .check(conn, config, site, stash)
        .await
      }

      Http => HttpHandler { check: &self }.check(conn, config, site, stash).await,
      Tcp => TcpHandler { check: &self }.check(conn, config, site, stash).await,
      Udp => UdpHandler { check: &self }.check(conn, config, site, stash).await,
      Tls => TlsHandler { check: &self }.check(conn, config, site, stash).await,
      PlayStore => PlayStoreHandler { check: &self }.check(conn, config, site, stash).await,
      AppStore => AppStoreHandler { check: &self }.check(conn, config, site, stash).await,
      Whois => WhoisHandler { check: &self }.check(conn, config, site, stash).await,
      DeadManSwitch => {
        let last = DeadManSwitchLog::last(conn, self.id).await.unwrap_or_default();

        DeadManSwitchHandler { check: &self, last }.check(conn, config, site, stash).await
      }
      Unsupported => Err(anyhow!("unsupported check kind")),
    }
  }

  pub async fn group(&self, conn: &mut MySqlConnection) -> Option<Group> {
    match self.group_id {
      Some(id) => Group::by_id(conn, id).await.ok(),
      None => None,
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
        let outage = Outage::by_uuid(&mut *conn, &outage).await?;

        if let Some(alerter) = self.alerter(&mut *conn).await {
          alerter.webhook().alert(&mut *conn, &self, &outage).await?;
        }

        Ok(())
      };

      if let Err(err) = inner().await {
        log::error!("{:#}", err);
      }
    }
  }

  pub async fn sites(&self, conn: &mut MySqlConnection) -> Result<Vec<Site>> {
    let sites = sqlx::query_as::<_, Site>("SELECT check_id, slug FROM check_sites WHERE check_id = ?")
      .bind(self.id)
      .fetch_all(&mut *conn)
      .await?;

    Ok(sites)
  }

  pub async fn update_sites(&self, conn: &mut MySqlConnection, sites: &[String]) -> Result<()> {
    Site::insert(conn, self, sites).await?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration as StdDuration;

  use anyhow::Result;
  use uuid::Uuid;

  use super::{Check, CheckKind, Event};
  use crate::{config::CONTROLLER_ID, model::Duration, tests};

  #[tokio::test]
  async fn list() -> Result<()> {
    let pool = tests::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(None, None, "all()", None, None).await?;

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
    let pool = tests::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(Some(1), Some(Uuid::new_v4().to_string()), "enabled()", Some(true), None).await?;
    pool.create_check(Some(2), Some(Uuid::new_v4().to_string()), "enabled()", Some(false), None).await?;

    let checks = Check::enabled(&mut *conn).await?;

    assert_eq!(checks.len(), 1);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn by_uuid() -> Result<()> {
    let pool = tests::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(None, None, "by_uuid()", None, None).await?;

    let check = Check::by_uuid(&mut *conn, "dd9a531a-1b0b-4a12-bc09-e5637f916261").await?;

    assert_eq!(check.name, "by_uuid()");
    assert_eq!(check.kind, CheckKind::Tcp);
    assert_eq!(check.enabled, true);
    assert_eq!(check.interval.as_secs(), 10);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn by_id() -> Result<()> {
    let pool = tests::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(None, None, "by_id()", None, None).await?;

    let check = Check::by_id(&mut *conn, 1).await?;

    assert_eq!(check.name, "by_id()");
    assert_eq!(check.kind, CheckKind::Tcp);
    assert_eq!(check.enabled, true);
    assert_eq!(check.interval.as_secs(), 10);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn insert() -> Result<()> {
    let pool = tests::db_client().await?;
    let mut conn = pool.acquire().await?;

    let check = Check {
      id: 1,
      group_id: None,
      alerter_id: None,
      uuid: "dd9a531a-1b0b-4a12-bc09-e5637f916261".to_string(),
      name: "create()".to_string(),
      kind: CheckKind::Tcp,
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
    let pool = tests::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(None, None, "update()", None, None).await?;

    let update = Check {
      id: 1,
      group_id: None,
      alerter_id: None,
      uuid: "dd9a531a-1b0b-4a12-bc09-e5637f916261".to_string(),
      name: "new_update()".to_string(),
      kind: CheckKind::Tcp,
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
    let pool = tests::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(None, None, "delete()", None, None).await?;

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
    let pool = tests::db_client().await?;
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
      site: CONTROLLER_ID.to_string(),
      status: 0,
      message: "First event".to_string(),
      ..Default::default()
    };

    event.insert(&mut *conn, None).await?;

    tokio::time::sleep(StdDuration::from_secs(1)).await;

    event.message = "Last event".to_string();
    event.insert(&mut *conn, None).await?;

    let last = check.last_event_for_site(&mut *conn, CONTROLLER_ID).await?;

    assert_eq!(last.is_some(), true);
    assert_eq!(&last.unwrap().message, "Last event");

    pool.cleanup().await;

    Ok(())
  }
}
