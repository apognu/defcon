use anyhow::{Context, Result};
use sqlx::MySqlConnection;

use crate::{
  alerters::Webhook,
  api::types as api,
  model::{status::*, Alerter, Check, Outage},
};

#[derive(Debug, Serialize)]
struct Payload<'p> {
  #[serde(flatten)]
  pub level: Option<&'p str>,
  pub check: &'p Check,
  pub spec: api::Spec,
  pub outage: &'p Outage,
}

pub struct WebhookAlerter(pub Alerter);

#[async_trait]
impl Webhook for WebhookAlerter {
  async fn alert(&self, conn: &mut MySqlConnection, check: &Check, outage: &Outage) -> Result<()> {
    let level = match check.last_event(&mut *conn).await {
      Ok(Some(event)) => match event.status {
        OK => Some("ok"),
        CRITICAL => Some("critical"),
        WARNING => Some("warning"),
        _ => None,
      },

      _ => None,
    };

    let spec = check.spec(&mut *conn).await?;
    let payload = Payload { level, check, spec, outage };
    let client = reqwest::Client::new();

    client.post(&self.0.webhook).json(&payload).send().await.context("could not call alerter webhook")?;

    Ok(())
  }
}
