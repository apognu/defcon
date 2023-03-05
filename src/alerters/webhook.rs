use std::sync::Arc;

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
use sqlx::MySqlConnection;

use crate::{
  alerters::Webhook,
  api::types as api,
  config::Config,
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
  async fn alert(&self, _config: Arc<Config>, conn: &mut MySqlConnection, check: &Check, outage: &Outage) -> Result<()> {
    let url = match self.0.url {
      Some(ref url) => url,
      None => return Err(anyhow!("could not retrieve Pagerduty integration key")),
    };

    let level = match check.last_event(conn).await {
      Ok(Some(Event { status: OK, .. })) => Some("ok"),
      Ok(Some(Event { status: CRITICAL, .. })) => Some("critical"),
      Ok(Some(Event { status: WARNING, .. })) => Some("warning"),
      _ => None,
    };

    let spec = check.spec(conn).await?;
    let payload = Payload { level, check, spec, outage };

    let request = ureq::post(url);

    let request = match &self.0.username {
      Some(username) => {
        let password = self.0.password.clone().unwrap_or_default();
        let credentials = b64.encode(&format!("{username}:{}", password));

        request.set("authorization", &format!("Basic {credentials}"))
      }

      None => request,
    };

    request.send_json(&payload).context("could not call alerter webhook")?;

    Ok(())
  }
}
