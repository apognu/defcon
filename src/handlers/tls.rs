use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::MySqlConnection;
use ssl_expiration::SslExpiration;

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
    let expiration = SslExpiration::from_domain_name(&spec.domain).map_err(|err| anyhow!("{}", err)).context("could not fetch certificate")?;

    let (status, message) = if expiration.secs() as u64 > spec.window.as_secs() {
      (OK, String::new())
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
