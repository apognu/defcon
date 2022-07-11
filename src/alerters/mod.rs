mod pagerduty;
mod slack;
mod webhook;

use std::sync::Arc;

use anyhow::Result;
use sqlx::MySqlConnection;

pub use self::{pagerduty::PagerdutyAlerter, slack::SlackAlerter, webhook::WebhookAlerter};
use crate::{
  config::Config,
  model::{Check, Outage},
};

#[async_trait]
pub trait Webhook {
  async fn alert(&self, config: Arc<Config>, conn: &mut MySqlConnection, check: &Check, outage: &Outage) -> Result<()>;
}

#[derive(Debug, Deserialize)]
pub struct NoopAlerter;

#[async_trait]
impl Webhook for NoopAlerter {
  async fn alert(&self, _config: Arc<Config>, _conn: &mut MySqlConnection, _check: &Check, _outage: &Outage) -> Result<()> {
    Ok(())
  }
}
