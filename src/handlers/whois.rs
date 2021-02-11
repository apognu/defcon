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
  stash::Stash,
};

pub struct WhoisHandler<'h> {
  pub check: &'h Check,
}

#[async_trait]
impl<'h> Handler for WhoisHandler<'h> {
  type Spec = Whois;

  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>, site: &str, stash: Stash) -> Result<Event> {
    let spec = Whois::for_check(conn, self.check).await?;

    self.run(&spec, site, stash).await
  }

  async fn run(&self, spec: &Whois, site: &str, _stash: Stash) -> Result<Event> {
    let attribute = spec.attribute.clone().unwrap_or_else(|| "registry expiry date".to_string());

    let mut whois = WhoisClient::new();
    let info = whois.get_whois_kv(&spec.domain).map_err(|err| anyhow!(err)).context("could not get information")?;
    let expiration = info.get(&attribute).ok_or_else(|| anyhow!("expiry date not found"))?;

    let now = Utc::now().naive_utc();
    let expires_in = NaiveDateTime::parse_from_str(expiration, "%Y-%m-%dt%H:%M:%Sz")
      .context("could not parse expiry date")?
      .signed_duration_since(now);

    let (status, message) = if spec.window.as_secs() < expires_in.num_seconds() as u64 {
      (OK, format!("domain is expiring in {} days", expires_in.num_days()))
    } else {
      (WARNING, format!("domain is expiring in {} days", expires_in.num_days()))
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

  use super::{Handler, WhoisHandler};
  use crate::{
    config::CONTROLLER_ID,
    model::{specs::Whois, status::*, Check, Duration},
    stash::Stash,
  };

  #[tokio::test]
  async fn handler_whois_ok() {
    let handler = WhoisHandler { check: &Check::default() };
    let spec = Whois {
      id: 0,
      check_id: 0,
      domain: "github.com".to_string(),
      attribute: None,
      window: Duration::try_from("90 days").unwrap(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[tokio::test]
  async fn handler_whois_warning() {
    let handler = WhoisHandler { check: &Check::default() };
    let spec = Whois {
      id: 0,
      check_id: 0,
      domain: "github.com".to_string(),
      attribute: None,
      window: Duration::try_from("10 years").unwrap(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, WARNING);
  }

  #[tokio::test]
  async fn handler_whois_attribute_warning() {
    let handler = WhoisHandler { check: &Check::default() };
    let spec = Whois {
      id: 0,
      check_id: 0,
      domain: "france.fr".to_string(),
      attribute: Some("expiry date".to_string()),
      window: Duration::try_from("100 years").unwrap(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, WARNING);
  }

  #[tokio::test]
  async fn handler_whois_invalid() {
    let handler = WhoisHandler { check: &Check::default() };
    let spec = Whois {
      id: 0,
      check_id: 0,
      domain: "example.com".to_string(),
      attribute: None,
      window: Duration::try_from("10 years").unwrap(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Err(_)));
  }

  #[tokio::test]
  async fn handler_whois_error() {
    let handler = WhoisHandler { check: &Check::default() };
    let spec = Whois {
      id: 0,
      check_id: 0,
      domain: "be83fb82-1203-49d0-8f88-c25cb42b2ef0.com".to_string(),
      attribute: None,
      window: Duration::try_from("10 years").unwrap(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Err(_)));
  }
}
