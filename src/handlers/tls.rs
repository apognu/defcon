use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::MySqlConnection;
use ssl_expiration2::SslExpiration;

use crate::{
  config::Config,
  handlers::Handler,
  model::{specs::Tls, status::*, Check, Event},
};

pub struct TlsHandler<'h> {
  pub check: &'h Check,
}

#[async_trait]
impl<'h> Handler for TlsHandler<'h> {
  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>) -> Result<Event> {
    let spec = Tls::for_check(conn, self.check).await.context("no spec found")?;

    self.run(spec).await
  }
}

impl<'h> TlsHandler<'h> {
  async fn run(&self, spec: Tls) -> Result<Event> {
    let expiration = SslExpiration::from_domain_name(&spec.domain).map_err(|err| anyhow!("{}", err)).context("could not fetch certificate")?;

    let (status, message) = if expiration.secs() > 0 && expiration.secs() as u64 > spec.window.as_secs() {
      (OK, format!("Expires in {} {} / Window is {}", expiration.secs(), expiration.days(), spec.window.as_secs()))
    } else {
      (CRITICAL, format!("TLS certificate for {} expires in {} days", spec.domain, expiration.days()))
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

#[cfg(test)]
mod tests {
  use std::convert::TryFrom;

  use tokio_test::*;

  use super::TlsHandler;
  use crate::model::{specs::Tls, status::*, Check, Duration};

  #[tokio::test]
  async fn handler_tls_ok() {
    let handler = TlsHandler { check: &Check::default() };
    let spec = Tls {
      id: 0,
      check_id: 0,
      domain: "letsencrypt.org".to_string(),
      window: Duration::try_from("0 days").unwrap(),
    };

    let result = handler.run(spec).await;
    assert_ok!(&result);

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[tokio::test]
  async fn handler_tls_critical() {
    let handler = TlsHandler { check: &Check::default() };
    let spec = Tls {
      id: 0,
      check_id: 0,
      domain: "letsencrypt.org".to_string(),
      window: Duration::try_from("91 days").unwrap(),
    };

    let result = handler.run(spec).await;
    assert_ok!(&result);

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
    assert_eq!(result.message.starts_with("TLS certificate for letsencrypt.org"), true);
  }

  #[tokio::test]
  async fn handler_tls_expired() {
    let handler = TlsHandler { check: &Check::default() };
    let spec = Tls {
      id: 0,
      check_id: 0,
      domain: "expired.badssl.com".to_string(),
      window: Duration::from(1),
    };

    let result = handler.run(spec).await;
    assert_ok!(&result);

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
    assert_eq!(result.message.starts_with("TLS certificate for expired.badssl.com expires in"), true);
  }

  #[tokio::test]
  async fn handler_tls_invalid() {
    let handler = TlsHandler { check: &Check::default() };
    let spec = Tls {
      id: 0,
      check_id: 0,
      domain: "*".to_string(),
      window: Duration::from(1),
    };

    let result = handler.run(spec).await;

    assert_err!(&result);
  }
}
