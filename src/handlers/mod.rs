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

pub async fn handle_event(conn: &mut MySqlConnection, site: &str, event: &Event, check: &Check, inhibitor: Option<Inhibitor>) -> Result<()> {
  let outage = SiteOutage::insert(&mut *conn, &check, &event).await.ok().flatten();

  event.insert(&mut *conn, outage.as_ref(), site).await?;

  if let Some(mut inhibitor) = inhibitor {
    inhibitor.release(site, &check.uuid);
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

  if event.status == 0 {
    kvlog!(Debug, "passed", {
      "site" => site,
      "kind" => check.kind,
      "check" => check.uuid,
      "name" => check.name,
      "message" => event.message
    });
  } else {
    kvlog!(Debug, "failed", {
      "site" => site,
      "kind" => check.kind,
      "check" => check.uuid,
      "name" => check.name,
      "message" => event.message
    });
  }

  Ok(())
}
