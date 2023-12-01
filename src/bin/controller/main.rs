#![feature(async_closure, try_find)]

#[macro_use]
extern crate anyhow;

mod cleaner;
mod deadmanswitch;
mod handler;
mod util;

use std::{env, net::SocketAddr, process, sync::Arc, time::Instant};

use anyhow::{Context, Result};
use futures::future;
use humantime::format_duration;
use kvlogger::*;
use sqlx::{
  mysql::{MySql, MySqlPoolOptions},
  Pool,
};

use defcon::{
  api::{self, auth::Keys},
  config::{Config, PUBLIC_KEY},
  inhibitor::Inhibitor,
  model::migrations,
  stash::Stash,
};

#[tokio::main]
async fn main() -> Result<()> {
  Config::set_log_level()?;

  let config = Config::parse()?;
  let keys = match &*PUBLIC_KEY {
    Some(key) => Some(Keys::new_public(key).context("public key should be ECDSA in PEM format")?),
    None => None,
  };

  let dsn = env::var("DSN")?;
  let pool = MySqlPoolOptions::new().max_connections(20).connect(&dsn).await?;

  if let (true, _) = migrations::migrate(&dsn, false)? {
    return Ok(());
  }

  match util::process(config.clone(), pool.clone()).await {
    Ok(true) => process::exit(0),
    Ok(false) => {}
    Err(err) => return Err(err),
  }

  if !config.handler.enable && !config.cleaner.enable && !config.api.enable {
    return Err(anyhow!("all processes disabled, aborting"));
  }

  if config.handler.enable {
    tokio::spawn({
      let config = config.clone();
      let pool = pool.clone();

      async move {
        run_defcon(&pool, config).await;
      }
    });
  }

  if config.cleaner.enable {
    tokio::spawn({
      let config = config.clone();
      let pool = pool.clone();

      async move {
        run_cleaner(&pool, config).await;
      }
    });
  }

  if config.dms.enable {
    tokio::spawn({
      let config = config.clone();
      let pool = pool.clone();

      async move {
        deadmanswitch::run(pool, config).await.unwrap();
      }
    });
  }

  match config.api.enable {
    true => {
      if config.api.jwt_signing_key.is_empty() {
        return Err(anyhow!("the JWT signing key must be provided"));
      }

      if let Err(err) = run_api(pool, config, keys).await {
        log::error!("{:#}", err);
      }
    }

    false => future::pending().await,
  }

  Ok(())
}

async fn run_api(pool: Pool<MySql>, config: Arc<Config>, keys: Option<Keys>) -> Result<()> {
  kvlog!(Info, "starting api process", {
    "listen" => config.api.listen
  });

  let addr = SocketAddr::from((config.api.listen.ip(), config.api.listen.port()));
  let app = api::server(config, pool, keys);
  let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

  axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
    .await
    .context("could not launch api process")?;

  Ok(())
}

async fn run_defcon(pool: &Pool<MySql>, config: Arc<Config>) {
  kvlog!(Info, "starting handler process", {
    "interval" => format_duration(config.handler.interval),
    "spread" => config.handler.spread.map(format_duration).map(|s| s.to_string()).unwrap_or_default()
  });

  let stash = Stash::new();
  let inhibitor = Inhibitor::new();

  loop {
    let next_tick_at = Instant::now() + config.handler.interval;

    tokio::spawn({
      let pool = pool.clone();
      let config = config.clone();
      let stash = stash.clone();
      let inhibitor = inhibitor.clone();

      async move {
        if let Err(err) = handler::tick(pool, config, stash, inhibitor).await {
          log::error!("{:#}", err);
        }
      }
    });

    tokio::time::sleep_until(next_tick_at.into()).await;
  }
}

async fn run_cleaner(pool: &Pool<MySql>, config: Arc<Config>) {
  kvlog!(Info, "starting cleaner process", {
    "interval" => format_duration(config.cleaner.interval),
    "threshold" => format_duration(config.cleaner.threshold)
  });

  loop {
    let config = config.clone();
    let next_tick_at = Instant::now() + config.cleaner.interval;

    tokio::spawn({
      let pool = pool.clone();

      async move {
        cleaner::tick(pool, config).await;
      }
    });

    tokio::time::sleep_until(next_tick_at.into()).await;
  }
}
