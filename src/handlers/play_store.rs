use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::MySqlConnection;

use crate::{
  config::Config,
  handlers::Handler,
  model::{specs::PlayStore, status::*, Check, Event},
  stash::Stash,
};

pub struct PlayStoreHandler<'h> {
  pub check: &'h Check,
}

#[async_trait]
impl Handler for PlayStoreHandler<'_> {
  type Spec = PlayStore;

  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>, site: &str, stash: Stash) -> Result<Event> {
    let spec = PlayStore::for_check(conn, self.check).await.context("no spec found")?;

    self.run(&spec, site, stash).await
  }

  async fn run(&self, spec: &PlayStore, site: &str, _stash: Stash) -> Result<Event> {
    let url = format!("https://play.google.com/store/apps/details?id={}", spec.app_id);
    let response = ureq::get(&url).call();

    let (status, message) = if response.is_ok() {
      (OK, String::new())
    } else {
      (CRITICAL, format!("Android app {} missing", spec.app_id))
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
  use super::{Handler, PlayStoreHandler};
  use crate::{
    config::CONTROLLER_ID,
    model::{specs::PlayStore, status::*, Check},
    stash::Stash,
  };

  #[tokio::test]
  async fn handler_play_store_ok() {
    let handler = PlayStoreHandler { check: &Check::default() };
    let spec = PlayStore {
      id: 0,
      check_id: 0,
      app_id: "com.google.android.apps.maps".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[tokio::test]
  async fn handler_play_store_critical() {
    let handler = PlayStoreHandler { check: &Check::default() };
    let spec = PlayStore {
      id: 0,
      check_id: 0,
      app_id: "29c4e9c3-c6f8-47d7-a64c-004e463d3aa8".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
  }
}
