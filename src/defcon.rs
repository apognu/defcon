use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};
use kvlogger::*;
use rand::Rng;
use sqlx::{MySql, Pool};

use crate::{
  config::Config,
  inhibitor::Inhibitor,
  model::{Check, Outage},
};

pub async fn tick(pool: Pool<MySql>, config: Arc<Config>, inhibitor: Inhibitor) -> Result<()> {
  let checks = {
    let mut conn = pool.acquire().await.context("could not retrieve database connection")?;

    Check::enabled(&mut conn).await
  };

  if let Ok(checks) = checks {
    let mut rng = rand::thread_rng();

    for check in checks {
      if inhibitor.inhibited(&check.uuid) {
        continue;
      }

      let spread = config.handler_spread.map(|duration| rng.gen_range(0..duration.as_millis() as u64));

      tokio::spawn({
        let config = config.clone();
        let pool = pool.clone();
        let mut inhibitor = inhibitor.clone();

        async move {
          let inner = async move || -> Result<()> {
            let should_run = {
              let mut conn = pool.acquire().await.context("could not retrieve database connection")?;

              check.stale(&mut *conn).await
            };

            if should_run {
              inhibitor.inhibit(&check.uuid);

              if let Some(spread) = spread {
                tokio::time::delay_for(Duration::from_millis(spread)).await
              }

              run(pool, config, check, inhibitor).await?;
            }

            Ok(())
          };

          if let Err(err) = inner().await {
            crate::log_error(&err);
          }
        }
      });
    }
  }

  Ok(())
}

async fn run(pool: Pool<MySql>, config: Arc<Config>, check: Check, mut inhibitor: Inhibitor) -> Result<()> {
  let inner = async move || -> Result<()> {
    use crate::handlers::*;

    let mut conn = pool.acquire().await.context("could not retrieve database connection")?;
    let handler = check.handler();

    match handler.check(&mut *conn, config).await {
      Err(err) => {
        inhibitor.inhibit_for(&check.uuid, *check.interval);

        kvlog!(Error, format!("{}: {}", err, err.root_cause()), {
          "kind" => check.kind,
          "check" => check.uuid,
          "name" => check.name
        });
      }

      Ok(event) => {
        let outage = Outage::insert(&mut conn, &check, &event).await.ok().flatten();
        Event::insert(&mut conn, &event, outage.as_ref()).await?;

        inhibitor.release(&check.uuid);

        if event.status == 0 {
          kvlog!(Debug, "passed", {
            "kind" => check.kind,
            "check" => check.uuid,
            "name" => check.name,
            "message" => event.message
          });
        } else {
          kvlog!(Debug, "failed", {
            "kind" => check.kind,
            "check" => check.uuid,
            "name" => check.name,
            "message" => event.message
          });
        }
      }
    }

    Ok(())
  };

  inner().await
}
