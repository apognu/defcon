use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};
use kvlogger::*;
use rand::Rng;
use sqlx::{MySql, Pool};

use defcon::{config::Config, handlers, inhibitor::Inhibitor, model::Check};

pub async fn tick(pool: Pool<MySql>, config: Arc<Config>, inhibitor: Inhibitor) -> Result<()> {
  let checks = {
    let mut conn = pool.acquire().await.context("could not retrieve database connection")?;

    Check::stale(&mut conn, "@controller").await
  };

  if let Ok(checks) = checks {
    let mut rng = rand::thread_rng();

    for check in checks {
      if inhibitor.inhibited("@controller", &check.uuid) {
        continue;
      }

      let spread = config.handler_spread.map(|duration| rng.gen_range(0..duration.as_millis() as u64));

      tokio::spawn({
        let config = config.clone();
        let pool = pool.clone();
        let mut inhibitor = inhibitor.clone();

        async move {
          let inner = async move || -> Result<()> {
            inhibitor.inhibit("@controller", &check.uuid);

            if let Some(spread) = spread {
              tokio::time::delay_for(Duration::from_millis(spread)).await
            }

            run(pool, config, check, inhibitor).await?;

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

async fn run(pool: Pool<MySql>, config: Arc<Config>, check: Check, mut inhibitor: Inhibitor) -> Result<()> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection")?;

  match check.run(&mut *conn, config, "@controller").await {
    Ok(event) => handlers::handle_event(&mut conn, &event, &check, Some(inhibitor)).await?,

    Err(err) => {
      inhibitor.inhibit_for("@controller", &check.uuid, *check.interval);

      kvlog!(Error, format!("{}: {}", err, err.root_cause()), {
        "kind" => check.kind,
        "check" => check.uuid,
        "name" => check.name
      });
    }
  }

  Ok(())
}
