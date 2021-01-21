use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{NaiveDateTime, Utc};
use sqlx::MySqlConnection;
use whois2::Client as WhoisClient;

use crate::{
  config::Config,
  handlers::Handler,
  model::{specs::Whois, status::*, Check, Event},
};

pub struct WhoisHandler<'h> {
  pub check: &'h Check,
}

#[async_trait]
impl<'h> Handler for WhoisHandler<'h> {
  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>) -> Result<Event> {
    let spec = Whois::for_check(conn, self.check).await?;
    let attribute = spec.attribute.unwrap_or_else(|| "registry expiry date".to_string());

    let mut whois = WhoisClient::new();
    let info = whois.get_whois_kv(&spec.domain).map_err(|err| anyhow!(err)).context("could not get information")?;
    let expiration = info.get(&attribute).ok_or_else(|| anyhow!("expiry date not found"))?;

    let now = Utc::now().naive_utc();
    let expires_in = NaiveDateTime::parse_from_str(expiration, "%Y-%m-%dt%H:%M:%Sz")
      .context("could not parse expiry date")?
      .signed_duration_since(now)
      .num_days();

    let (status, message) = if (spec.window as i64) < expires_in {
      (OK, format!("domain is expiring in {} days", expires_in))
    } else {
      (WARNING, format!("domain is expiring in {} days", expires_in))
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
