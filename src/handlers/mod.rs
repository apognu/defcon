mod app_store;
mod deadmanswitch;
mod dns;
mod http;
#[cfg(feature = "ping")]
mod ping;
mod play_store;
#[cfg(feature = "python")]
mod python;
mod tcp;
mod tls;
mod udp;
mod whois;

use kvlogger::*;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use sqlx::MySqlConnection;

#[cfg(feature = "ping")]
pub use crate::handlers::ping::PingHandler;
#[cfg(feature = "python")]
pub use crate::handlers::python::PythonHandler;
pub use crate::{
  config::Config,
  handlers::{
    app_store::AppStoreHandler, deadmanswitch::DeadManSwitchHandler, dns::DnsHandler, http::HttpHandler, play_store::PlayStoreHandler, tcp::TcpHandler, tls::TlsHandler, udp::UdpHandler,
    whois::WhoisHandler,
  },
  inhibitor::Inhibitor,
  model::{Check, Event, Outage, SiteOutage, Timeline},
  stash::Stash,
};

#[async_trait]
pub trait Handler: Send {
  type Spec: crate::model::specs::SpecMeta;

  async fn check(&self, conn: &mut MySqlConnection, config: Arc<Config>, site: &str, stash: Stash) -> Result<Event>;
  async fn run(&self, spec: &Self::Spec, site: &str, stash: Stash) -> Result<Event>;
}

pub async fn handle_event(config: Arc<Config>, conn: &mut MySqlConnection, event: &Event, check: &Check, inhibitor: Option<Inhibitor>) -> Result<()> {
  let title = if event.status == 0 { "check passed" } else { "check failed" };

  kvlog!(Debug, title, {
    "site" => event.site,
    "kind" => check.kind,
    "check" => check.uuid,
    "name" => check.name,
    "message" => event.message
  });

  let outage = SiteOutage::insert(&mut *conn, check, event).await.ok().flatten();

  event.insert(&mut *conn, outage.as_ref()).await?;

  if let Some(mut inhibitor) = inhibitor {
    inhibitor.release(&event.site, &check.uuid).await;
  }

  if let Some(outage) = outage {
    let payload = json!({
      "outage": {
        "site": event.site
      }
    });

    match outage.ended_on {
      None => {
        let site_outages = SiteOutage::all_for_check(&mut *conn, check).await?;

        if SiteOutage::count_for_check(conn, check).await? >= check.site_threshold as i64 {
          Outage::confirm(config, conn, check, site_outages).await?;
        }
      }

      Some(_) => {
        let global_outage = Outage::for_check_current(&mut *conn, check).await?;

        Timeline::new(global_outage.id, None, "site_outage_resolved", &payload.to_string()).insert(&mut *conn).await?;

        if SiteOutage::count_for_check(conn, check).await? < check.site_threshold as i64 {
          Outage::resolve(config, conn, check).await?;
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

    {
      let mut conn = pool.acquire().await?;

      pool.create_check(None, None, "outages_are_created()", None, Some(&[CONTROLLER_ID, "eu-1"])).await?;

      let config = tests::config(false);
      let check = Check::by_id(&mut conn, 1).await?;
      let mut event = Event {
        check_id: 1,
        site: CONTROLLER_ID.to_string(),
        status: 1,
        message: "failure".to_string(),
        ..Default::default()
      };

      super::handle_event(config.clone(), &mut conn, &event, &check, None).await?;
      assert_eq!(SiteOutage::count_for_check(&mut conn, &check).await?, 0);
      super::handle_event(config.clone(), &mut conn, &event, &check, None).await?;
      assert_eq!(SiteOutage::count_for_check(&mut conn, &check).await?, 1);

      assert!(matches!(Outage::for_check_current(&mut conn, &check).await, Err(_)));

      event.site = "eu-1".to_string();

      super::handle_event(config.clone(), &mut conn, &event, &check, None).await?;
      assert_eq!(SiteOutage::current(&mut conn).await?.len(), 1);
      super::handle_event(config.clone(), &mut conn, &event, &check, None).await?;
      assert_eq!(SiteOutage::current(&mut conn).await?.len(), 2);

      assert!(matches!(Outage::for_check_current(&mut conn, &check).await, Ok(_)));
    }

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn outages_are_resolved() -> Result<()> {
    let pool = tests::db_client().await?;

    {
      let mut conn = pool.acquire().await?;

      pool.create_check(None, None, "outages_are_resolved()", None, None).await?;

      let config = tests::config(false);
      let check = Check::by_id(&mut conn, 1).await?;
      let mut event = Event {
        check_id: 1,
        site: CONTROLLER_ID.to_string(),
        status: 1,
        message: "failure".to_string(),
        ..Default::default()
      };

      super::handle_event(config.clone(), &mut conn, &event, &check, None).await?;
      super::handle_event(config.clone(), &mut conn, &event, &check, None).await?;

      event.status = 0;

      super::handle_event(config.clone(), &mut conn, &event, &check, None).await?;
      super::handle_event(config.clone(), &mut conn, &event, &check, None).await?;

      assert!(matches!(Outage::for_check_current(&mut conn, &check).await, Err(_)));
    }

    pool.cleanup().await;

    Ok(())
  }
}
