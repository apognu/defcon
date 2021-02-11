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
  type Spec = Tls;

  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>, site: &str) -> Result<Event> {
    let spec = Tls::for_check(conn, self.check).await.context("no spec found")?;

    self.run(&spec, site).await
  }

  async fn run(&self, spec: &Tls, site: &str) -> Result<Event> {
    let expiration = SslExpiration::from_domain_name(&spec.domain).map_err(|err| anyhow!("{}", err)).context("could not fetch certificate")?;

    let (status, message) = if expiration.secs() > 0 && expiration.secs() as u64 > spec.window.as_secs() {
      (OK, format!("Certificate expires in {} days", expiration.days()))
    } else {
      (CRITICAL, format!("Certificate expires in {} days", expiration.days()))
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
  use std::convert::TryFrom;

  use super::{Handler, TlsHandler};
  use crate::{
    config::CONTROLLER_ID,
    model::{specs::Tls, status::*, Check, Duration},
  };

  #[tokio::test]
  async fn handler_tls_ok() {
    let handler = TlsHandler { check: &Check::default() };
    let spec = Tls {
      id: 0,
      check_id: 0,
      domain: "letsencrypt.org".to_string(),
      window: Duration::try_from("0 days").unwrap(),
    };

    let result = handler.run(&spec, CONTROLLER_ID).await;
    assert!(matches!(&result, Ok(_)));

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

    let result = handler.run(&spec, CONTROLLER_ID).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
    assert_eq!(result.message.starts_with("Certificate expires in"), true);
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

    let result = handler.run(&spec, CONTROLLER_ID).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
    assert_eq!(result.message.starts_with("Certificate expires in"), true);
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

    let result = handler.run(&spec, CONTROLLER_ID).await;
    assert!(matches!(&result, Err(_)));
  }
}
