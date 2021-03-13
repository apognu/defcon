use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::MySqlConnection;

use crate::{
  config::Config,
  handlers::Handler,
  model::{specs::AppStore, status::*, Check, Event},
  stash::Stash,
};

pub struct AppStoreHandler<'h> {
  pub check: &'h Check,
}

#[derive(Deserialize)]
struct AppStoreResponse {
  #[serde(rename = "resultCount")]
  results: i32,
}

#[async_trait]
impl<'h> Handler for AppStoreHandler<'h> {
  type Spec = AppStore;

  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>, site: &str, stash: Stash) -> Result<Event> {
    let spec = AppStore::for_check(conn, self.check).await.context("no spec found for check {}")?;

    self.run(&spec, site, stash).await
  }

  async fn run(&self, spec: &AppStore, site: &str, _stash: Stash) -> Result<Event> {
    let url = format!("https://itunes.apple.com/lookup?bundleId={}", spec.bundle_id);
    let response = reqwest::get(&url).await.context("did not receive a valid response")?;

    if response.status().as_u16() != 200 {
      return Err(anyhow!("did not receive a valid response"));
    }

    let response: AppStoreResponse = response.json().await.context("did not receive a valid response")?;

    let (status, message) = if response.results > 0 {
      (OK, String::new())
    } else {
      (CRITICAL, format!("iOS app {} missing", spec.bundle_id))
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
  use super::{AppStoreHandler, Handler};
  use crate::{
    config::CONTROLLER_ID,
    model::{specs::AppStore, status::*, Check},
    stash::Stash,
  };

  #[tokio::test]
  async fn handler_app_store_ok() {
    let handler = AppStoreHandler { check: &Check::default() };
    let spec = AppStore {
      id: 0,
      check_id: 0,
      bundle_id: "com.apple.Maps".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[tokio::test]
  async fn handler_app_store_missing() {
    let handler = AppStoreHandler { check: &Check::default() };
    let spec = AppStore {
      id: 0,
      check_id: 0,
      bundle_id: "2e0a5188-7220-41bf-b684-82d6a54b868a".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
  }
}
