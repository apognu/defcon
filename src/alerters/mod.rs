mod pagerduty;
mod slack;
mod webhook;

use anyhow::Result;
use sqlx::MySqlConnection;

pub use self::{pagerduty::PagerdutyAlerter, slack::SlackAlerter, webhook::WebhookAlerter};
use crate::model::{Check, Outage};

#[async_trait]
pub trait Webhook {
  async fn alert(&self, conn: &mut MySqlConnection, check: &Check, outage: &Outage) -> Result<()>;
}

#[derive(Debug, Deserialize)]
pub struct NoopAlerter;

#[async_trait]
impl Webhook for NoopAlerter {
  async fn alert(&self, _conn: &mut MySqlConnection, _check: &Check, _outage: &Outage) -> Result<()> {
    Ok(())
  }
}
