mod slack;
mod webhook;

use anyhow::{Context, Result};
use kvlogger::*;
use sqlx::MySqlConnection;

pub use self::{slack::SlackAlerter, webhook::WebhookAlerter};
use crate::model::{Check, SiteOutage};

#[async_trait]
pub trait Webhook {
  async fn alert(&self, conn: &mut MySqlConnection, check: &Check, outage: &SiteOutage) -> Result<()>;
}

#[derive(Debug, Deserialize)]
pub struct NoopAlerter;

#[async_trait]
impl Webhook for NoopAlerter {
  async fn alert(&self, conn: &mut MySqlConnection, check: &Check, outage: &SiteOutage) -> Result<()> {
    let down = outage.ended_on.is_none();

    // TODO: actual site and add something like **any site**
    let event = check
      .last_event(conn, "@controller")
      .await
      .context("could not find outage event")?
      .context("could not find outage event")?;

    let message = if down { "outage started" } else { "outage resolved" };

    kvlog!(Info, message, {
      "check" => check.uuid,
      "outage" => outage.uuid,
      "since" => outage.started_on.map(|dt| dt.to_string()).unwrap_or_else(|| "-".to_string()),
      "message" => event.message
    });

    Ok(())
  }
}
