use std::{
  net::{TcpStream, ToSocketAddrs},
  sync::Arc,
  time::Duration,
};

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::MySqlConnection;

use crate::{
  config::Config,
  handlers::Handler,
  model::{specs::Tcp, status::*, Check, Event},
};

pub struct TcpHandler<'h> {
  pub check: &'h Check,
}

#[async_trait]
impl<'h> Handler for TcpHandler<'h> {
  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>) -> Result<Event> {
    let spec = Tcp::for_check(conn, self.check).await.context("no spec found")?;
    let timeout = Duration::from_secs(spec.timeout.unwrap_or(5) as u64);

    let addr = format!("{}:{}", spec.host, spec.port);
    let addr = addr.to_socket_addrs().context("could not parse host")?.next().ok_or_else(|| anyhow!("could not parse host"))?;

    let (status, message) = match TcpStream::connect_timeout(&addr, timeout) {
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
