use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};
use kvlogger::*;
use rand::Rng;
use sqlx::{MySql, Pool};

use defcon::{
  config::{Config, CONTROLLER_ID},
  handlers,
  inhibitor::Inhibitor,
  model::Check,
  stash::Stash,
};

pub async fn tick(pool: Pool<MySql>, config: Arc<Config>, stash: Stash, inhibitor: Inhibitor) -> Result<()> {
  let checks = {
    let mut conn = pool.acquire().await.context("could not retrieve database connection")?;

    Check::stale(&mut conn, CONTROLLER_ID).await
  };

  if let Ok(checks) = checks {
    for check in checks {
      if inhibitor.inhibited(CONTROLLER_ID, &check.uuid).await {
        continue;
      }

      let spread = config.handler_spread.map(|duration| rand::thread_rng().gen_range(0..duration.as_millis() as u64));

      tokio::spawn({
        let config = config.clone();
        let pool = pool.clone();
        let stash = stash.clone();
        let mut inhibitor = inhibitor.clone();

        async move {
          let inner = async move || -> Result<()> {
            inhibitor.inhibit(CONTROLLER_ID, &check.uuid).await;

            if let Some(spread) = spread {
              tokio::time::sleep(Duration::from_millis(spread)).await
            }

            run(pool, config, check, stash, inhibitor).await?;

            Ok(())
          };

          if let Err(err) = inner().await {
            log::error!("{:#}", err);
          }
        }
      });
    }
  }

  Ok(())
}

async fn run(pool: Pool<MySql>, config: Arc<Config>, check: Check, stash: Stash, mut inhibitor: Inhibitor) -> Result<()> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection")?;

  match check.run(&mut *conn, config, CONTROLLER_ID, stash).await {
    Ok(event) => handlers::handle_event(&mut conn, &event, &check, Some(inhibitor)).await?,

    Err(err) => {
      inhibitor.inhibit_for(CONTROLLER_ID, &check.uuid, *check.interval).await;

      kvlog!(Error, format!("{}: {}", err, err.root_cause()), {
        "kind" => check.kind,
        "check" => check.uuid,
        "name" => check.name
      });
    }
  }

  Ok(())
}
