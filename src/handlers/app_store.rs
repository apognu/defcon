use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::MySqlConnection;

use crate::{
  config::Config,
  handlers::Handler,
  model::{specs::AppStore, status::*, Check, Event},
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
  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>) -> Result<Event> {
    let spec = AppStore::for_check(conn, self.check).await.context("no spec found for check {}")?;

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
      status,
      message,
      ..Default::default()
    };

    Ok(event)
  }
}
