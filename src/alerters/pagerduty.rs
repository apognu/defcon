use anyhow::{Context, Result};
use pagerduty_rs::{
  eventsv2async::EventsV2,
  types::{AlertResolve, AlertTrigger, AlertTriggerPayload, Event as PagerdutyEvent, Severity},
};
use sqlx::MySqlConnection;
use time::OffsetDateTime;

use crate::{
  alerters::Webhook,
  model::{status::*, Alerter, Check, Event, Outage},
};

pub struct PagerdutyAlerter(pub Alerter);

#[async_trait]
impl Webhook for PagerdutyAlerter {
  async fn alert(&self, conn: &mut MySqlConnection, check: &Check, outage: &Outage) -> Result<()> {
    let key = match self.0.password {
      Some(ref key) => key,
      None => return Err(anyhow!("could not retrieve Pagerduty integration key")),
    };

    let pagerduty = EventsV2::new(key.clone(), Some("defcon".to_string()))
      .map_err(|err| anyhow!(err.to_string()))
      .context("could not create Pagerduty alerter")?;

    let event = check.last_event(conn).await.context("could not find outage event")?;
    let down = outage.ended_on.is_none();

    let level = match event {
      Some(Event { status: CRITICAL, .. }) => Severity::Critical,
      Some(Event { status: WARNING, .. }) => Severity::Warning,
      _ => Severity::Critical,
    };

    let event = match down {
      true => PagerdutyEvent::AlertTrigger(AlertTrigger {
        payload: AlertTriggerPayload {
          summary: format!("{}: {}", check.name.clone(), event.map(|ev| ev.message).unwrap_or_default()),
          source: "defcon".to_owned(),
          timestamp: outage.started_on.and_then(|dt| OffsetDateTime::from_unix_timestamp(dt.timestamp()).ok()),
          severity: level,
          component: Some(check.name.clone()),
          group: check.group(&mut *conn).await.map(|group| group.name),
          class: Some(check.kind.to_string()),
          custom_details: None,
        },
        dedup_key: Some(outage.uuid.clone()),
        images: None,
        links: None,
        client: Some("Defcon".to_string()),
        // TODO: add the actual domain for the current Defcon instance
        client_url: Some(format!("/outages/{}", outage.uuid)),
      }),

      false => PagerdutyEvent::AlertResolve::<()>(AlertResolve { dedup_key: outage.uuid.clone() }),
    };

    pagerduty.event(event).await?;

    Ok(())
  }
}
