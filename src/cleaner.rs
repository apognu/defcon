use anyhow::{Context, Result};
use std::sync::Arc;

use chrono::{Duration, Utc};
use kvlogger::*;
use sqlx::{MySql, Pool};

use crate::{
  config::Config,
  model::{Event, Outage},
};

pub async fn tick(pool: Pool<MySql>, config: Arc<Config>) {
  let inner = async move || -> Result<()> {
    let threshold = Duration::from_std(config.cleaner_threshold)?;
    let epoch = Utc::now().naive_utc() - threshold;
    let mut txn = pool.begin().await?;

    let events = Event::delete_before(&mut txn, &epoch).await?;
    let outages = Outage::delete_before(&mut txn, &epoch).await?;

    txn.commit().await.context("could not commit transaction")?;

    if outages > 0 && events > 0 {
      kvlog!(Info, "cleaned database", {
        "outages" => outages,
        "events" => events
      });
    }

    Ok(())
  };

  if let Err(err) = inner().await {
    kvlog!(Error, "failed to run cleaner", { "error" => err });
  }
}
