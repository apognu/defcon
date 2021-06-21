#![feature(async_closure, try_find)]
#![deny(clippy::all)]
#![allow(clippy::unit_arg)]

#[macro_use]
extern crate anyhow;

mod cleaner;
mod deadmanswitch;
mod handler;

use std::{env, sync::Arc, time::Instant};

use anyhow::{Context, Result};
use futures::future;
use humantime::format_duration;
use kvlogger::*;
use rocket::config::Config as RocketConfig;
use sqlx::{
  mysql::{MySql, MySqlPoolOptions},
  Pool,
};

use defcon::{
  api::{self, auth::Keys, middlewares::ApiLogger},
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

  migrations::migrate(&dsn)?;

  if !config.handler && !config.cleaner && !config.api {
    return Err(anyhow!("all processes disabled, aborting"));
  }

  if config.handler {
    tokio::spawn({
      let config = config.clone();
      let pool = pool.clone();

      async move {
        run_defcon(&pool, config).await;
      }
    });
  }

  if config.cleaner {
    tokio::spawn({
      let config = config.clone();
      let pool = pool.clone();

      async move {
        run_cleaner(&pool, config).await;
      }
    });
  }

  if config.dms {
    tokio::spawn({
      let config = config.clone();
      let pool = pool.clone();

      async move {
        deadmanswitch::run(pool, config).await;
      }
    });
  }

  match config.api {
    true => {
      if let Err(err) = run_api(pool, config, keys).await {
        log::error!("{:#}", err);
      }
    }

    false => future::pending().await,
  }

  Ok(())
}

async fn run_api(pool: Pool<MySql>, config: Arc<Config>, keys: Option<Keys<'static>>) -> Result<()> {
  kvlog!(Info, "starting api process", {
    "listen" => config.api_listen
  });

  let provider = RocketConfig {
    address: config.api_listen.ip(),
    port: config.api_listen.port(),
    ..RocketConfig::release_default()
  };

  api::server(provider, pool, keys).attach(ApiLogger::new()).launch().await.context("could not launch api process")?;

  Ok(())
}

async fn run_defcon(pool: &Pool<MySql>, config: Arc<Config>) {
  kvlog!(Info, "starting handler process", {
    "interval" => format_duration(config.handler_interval),
    "spread" => config.handler_spread.map(format_duration).map(|s| s.to_string()).unwrap_or_default()
  });

  let stash = Stash::new();
  let inhibitor = Inhibitor::new();

  loop {
    let next_tick_at = Instant::now() + config.handler_interval;

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
    "interval" => format_duration(config.cleaner_interval),
    "threshold" => format_duration(config.cleaner_threshold)
  });

  loop {
    let config = config.clone();
    let next_tick_at = Instant::now() + config.cleaner_interval;

    tokio::spawn({
      let pool = pool.clone();

      async move {
        cleaner::tick(pool, config).await;
      }
    });

    tokio::time::sleep_until(next_tick_at.into()).await;
  }
}
