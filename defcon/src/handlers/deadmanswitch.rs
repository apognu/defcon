use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use humantime::format_duration;
use sqlx::MySqlConnection;

use crate::{
  config::Config,
  handlers::Handler,
  model::{specs::DeadManSwitch, status::*, Check, DeadManSwitchLog, Event},
  stash::Stash,
};

pub struct DeadManSwitchHandler<'h> {
  pub check: &'h Check,
  pub last: Option<DeadManSwitchLog>,
}

#[async_trait]
impl<'h> Handler for DeadManSwitchHandler<'h> {
  type Spec = DeadManSwitch;

  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>, site: &str, stash: Stash) -> Result<Event> {
    let spec = DeadManSwitch::for_check(conn, self.check).await.context("no spec found")?;

    self.run(&spec, site, stash).await
  }

  async fn run(&self, spec: &DeadManSwitch, site: &str, _stash: Stash) -> Result<Event> {
    match &self.last {
      None => Err(anyhow!("check has never run before")),

      Some(log) => {
        let now = Utc::now();
        let last_at = -log.created_at.unwrap().signed_duration_since(now);
        let max = Duration::from_std(spec.stale_after.0).unwrap_or_else(|_| Duration::max_value());

        let (status, message) = if last_at <= max {
          (OK, String::new())
        } else {
          (CRITICAL, format!("last check in was {} ago", format_duration(last_at.to_std().unwrap())))
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
  }
}
