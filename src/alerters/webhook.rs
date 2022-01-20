use anyhow::{Context, Result};
use sqlx::MySqlConnection;

use crate::{
  alerters::Webhook,
  api::types as api,
  model::{status::*, Alerter, Check, Event, Outage},
};

#[derive(Debug, Clone, Serialize)]
struct Payload<'p> {
  pub level: Option<&'p str>,
  #[serde(flatten)]
  pub check: &'p Check,
  pub spec: api::Spec,
  pub outage: &'p Outage,
}

pub struct WebhookAlerter(pub Alerter);

#[async_trait]
impl Webhook for WebhookAlerter {
  async fn alert(&self, conn: &mut MySqlConnection, check: &Check, outage: &Outage) -> Result<()> {
    let level = match check.last_event(&mut *conn).await {
      Ok(Some(Event { status: OK, .. })) => Some("ok"),
      Ok(Some(Event { status: CRITICAL, .. })) => Some("critical"),
      Ok(Some(Event { status: WARNING, .. })) => Some("warning"),
      _ => None,
    };

    let spec = check.spec(&mut *conn).await?;
    let payload = Payload { level, check, spec, outage };
    let client = reqwest::Client::new();

    let builder = client.post(&self.0.webhook);
    let builder = match &self.0.username {
      Some(username) => builder.basic_auth(username, self.0.password.as_ref()),
      None => builder,
    };

    builder.json(&payload).send().await.context("could not call alerter webhook")?;

    Ok(())
  }
}
