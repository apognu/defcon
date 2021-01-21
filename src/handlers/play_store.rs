use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::MySqlConnection;

use crate::{
  config::Config,
  handlers::Handler,
  model::{specs::PlayStore, status::*, Check, Event},
};

pub struct PlayStoreHandler<'h> {
  pub check: &'h Check,
}

#[async_trait]
impl<'h> Handler for PlayStoreHandler<'h> {
  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>) -> Result<Event> {
    let spec = PlayStore::for_check(conn, self.check).await.context("no spec found")?;
    let url = format!("https://play.google.com/store/apps/details?id={}", spec.app_id);

    let response = reqwest::get(&url).await.context("did not receive a valid response")?;

    let (status, message) = if response.status().as_u16() == 200 {
      (OK, String::new())
    } else {
      (CRITICAL, format!("Android app {} missing", spec.app_id))
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
