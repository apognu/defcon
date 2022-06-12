use std::{net::ToSocketAddrs, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::MySqlConnection;
use tokio::{net::TcpStream, time};

use crate::{
  config::Config,
  handlers::Handler,
  model::{specs::Tcp, status::*, Check, Duration, Event},
  stash::Stash,
};

pub struct TcpHandler<'h> {
  pub check: &'h Check,
}

#[async_trait]
impl<'h> Handler for TcpHandler<'h> {
  type Spec = Tcp;

  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>, site: &str, stash: Stash) -> Result<Event> {
    let spec = Tcp::for_check(conn, self.check).await.context("no spec found")?;

    self.run(&spec, site, stash).await
  }

  async fn run(&self, spec: &Tcp, site: &str, _stash: Stash) -> Result<Event> {
    let timeout = spec.timeout.unwrap_or_else(|| Duration::from(5));

    let addr = format!("{}:{}", spec.host, spec.port);
    let addr = addr.to_socket_addrs().context("could not parse host")?.next().ok_or_else(|| anyhow!("could not parse host"))?;

    let (status, message) = match time::timeout(*timeout, TcpStream::connect(&addr)).await {
      Ok(_) => (OK, String::new()),
      Err(err) => (CRITICAL, err.to_string()),
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
  use super::{Handler, TcpHandler};
  use crate::{
    config::CONTROLLER_ID,
    model::{specs::Tcp, status::*, Check, Duration},
    stash::Stash,
  };

  #[tokio::test]
  async fn handler_tcp_ok() {
    let handler = TcpHandler { check: &Check::default() };
    let spec = Tcp {
      id: 0,
      check_id: 0,
      host: "example.com".to_string(),
      port: 80,
      timeout: None,
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[tokio::test]
  async fn handler_tcp_critical() {
    let handler = TcpHandler { check: &Check::default() };
    let spec = Tcp {
      id: 0,
      check_id: 0,
      host: "github.com".to_string(),
      port: 81,
      timeout: Some(Duration::from(1)),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
    assert_eq!(result.message, "deadline has elapsed".to_string());
  }

  #[tokio::test]
  async fn handler_tcp_invalid() {
    let handler = TcpHandler { check: &Check::default() };
    let spec = Tcp {
      id: 0,
      check_id: 0,
      host: "300.300.300.300".to_string(),
      port: 81,
      timeout: None,
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Err(_)));
  }
}
