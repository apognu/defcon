use std::{net::ToSocketAddrs, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use async_trait::async_trait;
use caps::{Capability, CapSet};
use sqlx::MySqlConnection;
use surge_ping::{Client as PingClient, Config as PingConfig};

use crate::{
  config::Config,
  handlers::Handler,
  model::{specs::Ping, status::*, Check, Event},
  stash::Stash,
};

pub struct PingHandler<'h> {
  pub check: &'h Check,
}

#[async_trait]
impl<'h> Handler for PingHandler<'h> {
  type Spec = Ping;

  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>, site: &str, stash: Stash) -> Result<Event> {
    let spec = Ping::for_check(conn, self.check).await.context("no spec found")?;

    self.run(&spec, site, stash).await
  }

  async fn run(&self, spec: &Ping, site: &str, _stash: Stash) -> Result<Event> {
    #[cfg(target_os = "linux")]
    if !caps::has_cap(None, CapSet::Effective, Capability::CAP_NET_RAW)? {
      return Err(anyhow!("ping: missing CAP_NET_RAW capabilities"));
    }

    let host = format!("{}:{}", spec.host, 0)
      .to_socket_addrs()
      .context("could not parse host")?
      .next()
      .ok_or_else(|| anyhow!("could not parse host"))?;

    let client = PingClient::new(&PingConfig::default())?;
    let mut pinger = client.pinger(host.ip()).await;
    pinger.timeout(Duration::from_secs(5));

    let (status, message) = match pinger.ping(0).await {
      Ok(_) => (OK, String::new()),
      Err(err) => (CRITICAL, format!("could not ping {}: {}", spec.host, err)),
    };

    let event = Event {
      check_id: self.check.id,
      site: site.to_string(),
      status,
      message,
      ..Default::default()
    };

    Ok(event)
  }
}

#[cfg(test)]
mod tests {
  use super::{Handler, PingHandler};
  use crate::{
    config::CONTROLLER_ID,
    model::{specs::Ping, status::*, Check},
    stash::Stash,
  };

  #[tokio::test]
  async fn handler_ping_ok() {
    let handler = PingHandler { check: &Check::default() };
    let spec = Ping {
      id: 0,
      check_id: 0,
      host: "127.0.0.1".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[tokio::test]
  async fn handler_ping_unreachable() {
    let handler = PingHandler { check: &Check::default() };
    let spec = Ping {
      id: 0,
      check_id: 0,
      host: "1.2.3.4".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
  }
}
