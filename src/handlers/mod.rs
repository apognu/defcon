mod app_store;
mod dns;
mod http;
mod ping;
mod play_store;
mod tcp;
mod tls;
mod udp;
mod whois;

use kvlogger::*;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use sqlx::MySqlConnection;

pub use crate::{
  config::Config,
  handlers::{app_store::AppStoreHandler, dns::DnsHandler, http::HttpHandler, ping::PingHandler, play_store::PlayStoreHandler, tcp::TcpHandler, tls::TlsHandler, udp::UdpHandler, whois::WhoisHandler},
  inhibitor::Inhibitor,
  model::{Check, Event, Outage, SiteOutage},
};

#[async_trait]
pub trait Handler: Send {
  type Spec: crate::model::specs::SpecMeta;

  async fn check(&self, conn: &mut MySqlConnection, config: Arc<Config>, site: &str) -> Result<Event>;
  async fn run(&self, spec: &Self::Spec, site: &str) -> Result<Event>;
}

pub async fn handle_event(conn: &mut MySqlConnection, event: &Event, check: &Check, inhibitor: Option<Inhibitor>) -> Result<()> {
  if event.status == 0 {
    kvlog!(Debug, "check passed", {
      "site" => event.site,
      "kind" => check.kind,
      "check" => check.uuid,
      "name" => check.name,
      "message" => event.message
    });
  } else {
    kvlog!(Debug, "check failed", {
      "site" => event.site,
      "kind" => check.kind,
      "check" => check.uuid,
      "name" => check.name,
      "message" => event.message
    });
  }

  let outage = SiteOutage::insert(&mut *conn, &check, &event).await.ok().flatten();

  event.insert(&mut *conn, outage.as_ref()).await?;

  if let Some(mut inhibitor) = inhibitor {
    inhibitor.release(&event.site, &check.uuid);
  }

  if let Some(outage) = outage {
    match outage.ended_on {
      None => {
        if SiteOutage::count(conn, &check).await? >= check.site_threshold as i64 {
          Outage::confirm(conn, &check).await?;
        }
      }

      Some(_) => {
        if SiteOutage::count(conn, &check).await? < check.site_threshold as i64 {
          Outage::resolve(conn, check).await?;
        }
      }
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use anyhow::Result;

  use crate::{
    config::CONTROLLER_ID,
    model::{Check, Event, Outage, SiteOutage},
    tests,
  };

  #[tokio::test]
  async fn outages_are_created() -> Result<()> {
    let pool = tests::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(None, None, "outages_are_created()", None, Some(&[CONTROLLER_ID, "eu-1"])).await?;

    let check = Check::by_id(&mut *conn, 1).await?;
    let mut event = Event {
      check_id: 1,
      site: CONTROLLER_ID.to_string(),
      status: 1,
      message: "failure".to_string(),
      ..Default::default()
    };

    super::handle_event(&mut *conn, &event, &check, None).await?;
    assert_eq!(SiteOutage::count(&mut *conn, &check).await?, 0);
    super::handle_event(&mut *conn, &event, &check, None).await?;
    assert_eq!(SiteOutage::count(&mut *conn, &check).await?, 1);

    assert!(matches!(Outage::for_check(&mut *conn, &check).await, Err(_)));

    event.site = "eu-1".to_string();

    super::handle_event(&mut *conn, &event, &check, None).await?;
    assert_eq!(SiteOutage::current(&mut *conn).await?.len(), 1);
    super::handle_event(&mut *conn, &event, &check, None).await?;
    assert_eq!(SiteOutage::current(&mut *conn).await?.len(), 2);

    assert!(matches!(Outage::for_check(&mut *conn, &check).await, Ok(_)));

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn outages_are_resolved() -> Result<()> {
    let pool = tests::db_client().await?;
    let mut conn = pool.acquire().await?;

    pool.create_check(None, None, "outages_are_resolved()", None, None).await?;

    let check = Check::by_id(&mut *conn, 1).await?;
    let mut event = Event {
      check_id: 1,
      site: CONTROLLER_ID.to_string(),
      status: 1,
      message: "failure".to_string(),
      ..Default::default()
    };

    super::handle_event(&mut *conn, &event, &check, None).await?;
    super::handle_event(&mut *conn, &event, &check, None).await?;

    event.status = 0;

    super::handle_event(&mut *conn, &event, &check, None).await?;
    super::handle_event(&mut *conn, &event, &check, None).await?;

    assert!(matches!(Outage::for_check(&mut *conn, &check).await, Err(_)));

    pool.cleanup().await;

    Ok(())
  }
}
