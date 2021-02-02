use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use capabilities::{Capabilities, Capability, Flag};
use ekko::{Ekko, EkkoResponse};
use sqlx::MySqlConnection;

use crate::{
  config::Config,
  handlers::Handler,
  model::{specs::Ping, status::*, Check, Event},
};

pub struct PingHandler<'h> {
  pub check: &'h Check,
}

#[async_trait]
impl<'h> Handler for PingHandler<'h> {
  type Spec = Ping;

  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>, site: &str) -> Result<Event> {
    let spec = Ping::for_check(conn, self.check).await.context("no spec found")?;

    self.run(&spec, site).await
  }

  async fn run(&self, spec: &Ping, site: &str) -> Result<Event> {
    #[cfg(target_os = "linux")]
    if let Ok(caps) = Capabilities::from_current_proc() {
      if !caps.check(Capability::CAP_NET_RAW, Flag::Effective) {
        return Err(anyhow!("ping: missing CAP_NET_RAW capabilities"));
      }
    }

    let mut ping = Ekko::with_target(&spec.host).context("ping: could not send echo request")?;

    let (status, message) = match ping.send(std::u8::MAX)? {
      EkkoResponse::DestinationResponse(data) => (OK, format!("ping successful in {}ms", data.elapsed.as_millis())),
      EkkoResponse::UnreachableResponse(_) => (CRITICAL, "host unreachable".to_string()),
      EkkoResponse::UnexpectedResponse(_) => (CRITICAL, "unexpected response".to_string()),
      EkkoResponse::ExceededResponse(_) => (CRITICAL, "timed out".to_string()),
      EkkoResponse::LackingResponse(_) => (CRITICAL, "no response".to_string()),
    };

    let event = Event {
      check_id: self.check.id,
      site: site.to_string(),
      status,
      message: format!("{}: {}", spec.host, message),
      ..Default::default()
    };

    Ok(event)
  }
}
