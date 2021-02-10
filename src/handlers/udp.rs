use std::{net::ToSocketAddrs, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::MySqlConnection;
use tokio::{net::UdpSocket, time};

use crate::{
  config::Config,
  handlers::Handler,
  model::{specs::Udp, status::*, Check, Duration, Event},
};

pub struct UdpHandler<'h> {
  pub check: &'h Check,
}

#[async_trait]
impl<'h> Handler for UdpHandler<'h> {
  type Spec = Udp;

  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>, site: &str) -> Result<Event> {
    let spec = Udp::for_check(conn, self.check).await?;

    self.run(&spec, site).await
  }

  async fn run(&self, spec: &Udp, site: &str) -> Result<Event> {
    let timeout = spec.timeout.unwrap_or_else(|| Duration::from(5));

    let addr = format!("{}:{}", spec.host, spec.port);
    let addr = addr.to_socket_addrs().context("could not parse host")?.next().ok_or_else(|| anyhow!("could not parse host"))?;

    let socket = UdpSocket::bind("0.0.0.0:0").await.context("could not open socket")?;
    let mut buf = [0; 1024];

    socket.connect(addr).await.context("could not connect socket")?;
    socket.send(&spec.message).await.context("could not send datagram")?;

    let (status, message) = match time::timeout(*timeout, socket.recv(&mut buf)).await {
      Ok(response) => match response {
        Ok(_) => match buf.windows(spec.content.len()).any(|window| window == *spec.content) {
          true => (OK, String::new()),
          false => (CRITICAL, "expected content not found".to_string()),
        },

        Err(err) => (CRITICAL, err.to_string()),
      },

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
  use std::{net::SocketAddr, time::Duration};

  use anyhow::Result;
  use tokio::net::UdpSocket;

  use super::{Handler, UdpHandler};
  use crate::{
    config::CONTROLLER_ID,
    model::{specs::Udp, status::*, Check},
  };

  async fn server(response: &[u8], timeout: bool) -> Result<SocketAddr> {
    let mut buf = Vec::with_capacity(32);
    let server = UdpSocket::bind("0.0.0.0:0").await?;
    let addr = server.local_addr()?;

    tokio::spawn({
      let response = response.to_vec();

      async move {
        let (_, peer) = server.recv_from(&mut buf).await.unwrap();

        if timeout {
          tokio::time::sleep(Duration::from_secs(2)).await;
        }

        server.send_to(&response, &peer).await.unwrap();
      }
    });

    Ok(addr)
  }

  #[tokio::test]
  async fn udp_ok() -> Result<()> {
    let addr = tokio::spawn(server("hello".as_bytes(), false)).await??;

    let handler = UdpHandler { check: &Check::default() };
    let spec = Udp {
      id: 0,
      check_id: 0,
      host: addr.ip().to_string(),
      port: addr.port(),
      timeout: Some(1.into()),
      message: "anything".as_bytes().into(),
      content: "hello".as_bytes().into(),
    };

    let result = handler.run(&spec, CONTROLLER_ID).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, OK);

    Ok(())
  }

  #[tokio::test]
  async fn udp_wrong_response() -> Result<()> {
    let addr = tokio::spawn(server("invalid".as_bytes(), false)).await??;

    let handler = UdpHandler { check: &Check::default() };
    let spec = Udp {
      id: 0,
      check_id: 0,
      host: addr.ip().to_string(),
      port: addr.port(),
      timeout: Some(1.into()),
      message: "anything".as_bytes().into(),
      content: "hello".as_bytes().into(),
    };

    let result = handler.run(&spec, CONTROLLER_ID).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);

    Ok(())
  }

  #[tokio::test]
  async fn udp_no_response() -> Result<()> {
    let addr = tokio::spawn(server("hello".as_bytes(), false)).await??;

    let handler = UdpHandler { check: &Check::default() };
    let spec = Udp {
      id: 0,
      check_id: 0,
      host: addr.ip().to_string(),
      port: addr.port() - 1,
      timeout: Some(1.into()),
      message: "anything".as_bytes().into(),
      content: "hello".as_bytes().into(),
    };

    let result = handler.run(&spec, CONTROLLER_ID).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);

    Ok(())
  }

  #[tokio::test]
  async fn udp_timeout() -> Result<()> {
    let addr = tokio::spawn(server("hello".as_bytes(), true)).await??;

    let handler = UdpHandler { check: &Check::default() };
    let spec = Udp {
      id: 0,
      check_id: 0,
      host: addr.ip().to_string(),
      port: addr.port() - 1,
      timeout: Some(1.into()),
      message: "anything".as_bytes().into(),
      content: "hello".as_bytes().into(),
    };

    let result = handler.run(&spec, CONTROLLER_ID).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);

    Ok(())
  }
}
