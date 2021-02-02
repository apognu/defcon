use chrono::Utc;

use anyhow::{Context, Result};
use slack_hook::{AttachmentBuilder, Field, PayloadBuilder, Section, Slack};
use sqlx::MySqlConnection;

use crate::{
  alerters::Webhook,
  model::{status::*, Alerter, Check, Event, SiteOutage},
};

const COLOR_UNKNOWN: &str = "#95a5a6";
const COLOR_OK: &str = "#00b894";
const COLOR_CRITICAL: &str = "#e17055";
const COLOR_WARNING: &str = "#e67e22";

pub struct SlackAlerter(pub Alerter);

#[async_trait]
impl Webhook for SlackAlerter {
  async fn alert(&self, conn: &mut MySqlConnection, check: &Check, outage: &SiteOutage) -> Result<()> {
    let slack = Slack::new(self.0.webhook.as_str()).map_err(|err| anyhow!(err.to_string()).context("could not create Slack alerter"))?;

    // TODO: add something like **any site**
    let event = check.last_event(conn, "@controller").await.context("could not find outage event")?;

    let spec = check.spec(conn).await.context("could not retrieve check spec")?;
    let down = outage.ended_on.is_none();

    let (level, color) = match event {
      Some(Event { status: OK, .. }) => ("(ok)", COLOR_OK),
      Some(Event { status: CRITICAL, .. }) => ("(critical)", COLOR_CRITICAL),
      Some(Event { status: WARNING, .. }) => ("(warning)", COLOR_WARNING),
      _ => ("", COLOR_UNKNOWN),
    };

    let event = event.unwrap();
    let meta = spec.meta();
    let fields = meta.fields().into_iter().map(|(k, v)| Field::new(k, v, Some(true)));

    let (color, title, description) = if down {
      (
        color,
        format!("{}: Outage started {} 🚨", check.name, level),
        format!("An uptime check for the following service failed.\n```{}```", event.message),
      )
    } else {
      (color, format!("{}: Outage recovered 👍", check.name), "Everything seems to be back to normal.".to_string())
    };

    let fields = vec![Field::new("Check name", check.name.clone(), Some(true)), Field::new("Check", meta.name(), Some(true))]
      .into_iter()
      .chain(fields)
      .collect();

    let attachments = vec![AttachmentBuilder::new(description)
      .title(title)
      .color(color)
      .fields(fields)
      .markdown_in([Section::Text].iter())
      .ts(&Utc::now().naive_utc())
      .build()
      .map_err(|err| anyhow!(err.to_string()).context("could not create Slack field"))?];

    let payload = PayloadBuilder::new()
      .username("Defcon")
      .icon_emoji(":mag:")
      .attachments(attachments)
      .build()
      .map_err(|_| anyhow!("could not create Slack alerter"))?;

    slack.send(&payload).map_err(|err| anyhow!(err.to_string()).context("could not send Slack notification"))?;

    Ok(())
  }
}
