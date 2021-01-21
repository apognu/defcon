use std::{net::ToSocketAddrs, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::MySqlConnection;
use std::time::Duration;
use tokio::{net::UdpSocket, time};

use crate::{
  config::Config,
  handlers::Handler,
  model::{specs::Udp, status::*, Check, Event},
};

pub struct UdpHandler<'h> {
  pub check: &'h Check,
}

#[async_trait]
impl<'h> Handler for UdpHandler<'h> {
  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>) -> Result<Event> {
    let spec = Udp::for_check(conn, self.check).await?;
    let timeout = Duration::from_secs(spec.timeout.unwrap_or(5) as u64);

    let addr = format!("{}:{}", spec.host, spec.port);
    let addr = addr.to_socket_addrs().context("could not parse host")?.next().ok_or_else(|| anyhow!("could not parse host"))?;

    let mut socket = UdpSocket::bind("0.0.0.0:0").await.context("could not open socket")?;
    let mut buf = [0; 1024];

    socket.connect(addr).await.context("could not connect socket")?;
    socket.send(&spec.message).await.context("could not send datagram")?;

    let (status, message) = match time::timeout(timeout, socket.recv(&mut buf)).await? {
      Ok(_) => match buf.windows(spec.content.len()).any(|window| window == *spec.content) {
        true => (OK, String::new()),
        false => (CRITICAL, "expected content not found".to_string()),
      },

      Err(err) => (1, err.to_string()),
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
