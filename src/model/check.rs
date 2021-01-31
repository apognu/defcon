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
  model::{specs, Alerter, CheckKind, Duration, Event},
};

use super::Outage;

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
  pub interval: Duration,
  pub passing_threshold: u8,
  pub failing_threshold: u8,
  #[serde(default = "ext::to_false")]
  pub silent: bool,
}

impl Check {
  pub async fn all(conn: &mut MySqlConnection) -> Result<Vec<Check>> {
    let checks = sqlx::query_as::<_, Check>(
      "
        SELECT id, uuid, alerter_id, name, enabled, kind, `interval`, passing_threshold, failing_threshold, silent
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
        SELECT id, uuid, alerter_id, name, enabled, kind, `interval`, passing_threshold, failing_threshold, silent
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
        SELECT id, uuid, alerter_id, name, enabled, kind, `interval`, passing_threshold, failing_threshold, silent
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
        SELECT id, uuid, alerter_id, name, enabled, kind, `interval`, passing_threshold, failing_threshold, silent
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
        INSERT INTO checks ( uuid, alerter_id, name, enabled, kind, `interval`, passing_threshold, failing_threshold, silent )
        VALUES ( ?, ?, ?, ?, ?, ?, ?, ?, ? )
      ",
    )
    .bind(&self.uuid)
    .bind(self.alerter_id)
    .bind(self.name)
    .bind(self.enabled)
    .bind(self.kind)
    .bind(self.interval)
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
        SET alerter_id = ?, name = ?, enabled = ?, kind = ?, `interval` = ?, passing_threshold = ?, failing_threshold = ?, silent = ?
        WHERE id = ?
      ",
    )
    .bind(self.alerter_id)
    .bind(self.name)
    .bind(self.enabled)
    .bind(self.kind)
    .bind(self.interval)
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

  pub async fn last_event(&self, conn: &mut MySqlConnection) -> Result<Option<Event>> {
    let event = sqlx::query_as::<_, Event>(
      "
        SELECT id, check_id, outage_id, status, message, created_at
        FROM events
        WHERE check_id = ?
        ORDER BY created_at DESC
        LIMIT 1
      ",
    )
    .bind(self.id)
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

  pub async fn stale(&self, conn: &mut MySqlConnection) -> bool {
    let event = self.last_event(&mut *conn).await.unwrap_or(None);

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
        let outage = Outage::by_uuid(&mut *conn, &outage).await?;

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
