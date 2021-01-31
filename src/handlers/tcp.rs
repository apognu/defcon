use std::{net::ToSocketAddrs, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::MySqlConnection;
use tokio::{net::TcpStream, time};

use crate::{
  config::Config,
  handlers::Handler,
  model::{specs::Tcp, status::*, Check, Duration, Event},
};

pub struct TcpHandler<'h> {
  pub check: &'h Check,
}

#[async_trait]
impl<'h> Handler for TcpHandler<'h> {
  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>) -> Result<Event> {
    let spec = Tcp::for_check(conn, self.check).await.context("no spec found")?;
    let timeout = spec.timeout.unwrap_or_else(|| Duration::from(5));

    let addr = format!("{}:{}", spec.host, spec.port);
    let addr = addr.to_socket_addrs().context("could not parse host")?.next().ok_or_else(|| anyhow!("could not parse host"))?;

    let (status, message) = match time::timeout(*timeout, TcpStream::connect(&addr)).await {
      Ok(_) => (OK, String::new()),
      Err(err) => (CRITICAL, err.to_string()),
    };

    let event = Event {
      check_id: self.check.id,
      status,
      message,
      ..Default::default()
    };

    Ok(event)
  }
}
