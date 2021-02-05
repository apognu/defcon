#![feature(async_closure, try_find)]
#![deny(clippy::all)]
#![allow(clippy::unit_arg)]

#[macro_use]
extern crate anyhow;

mod cleaner;
mod handler;

use std::{env, sync::Arc, time::Instant};

use anyhow::{Context, Error, Result};
use futures::future;
use lazy_static::lazy_static;
use rocket::config::Config as RocketConfig;
use sqlx::{
  mysql::{MySql, MySqlPoolOptions},
  Pool,
};

use defcon::{
  api::{self, auth::Keys, middlewares::ApiLogger},
  config::Config,
  inhibitor::Inhibitor,
  model::migrations,
};

lazy_static! {
  static ref PUBLIC_KEY: Vec<u8> = env::var("PUBLIC_KEY")
    .map(|key| format!("-----BEGIN PUBLIC KEY-----{}-----END PUBLIC KEY-----", key))
    .unwrap_or_default()
    .as_bytes()
    .to_vec();
}

pub fn log_error(err: &Error) {
  let desc = err.to_string();
  let cause = err.root_cause().to_string();

  if desc == cause {
    log::error!("{}", desc);
  } else {
    log::error!("{}: {}", desc, cause);
  }
}

#[tokio::main]
async fn main() -> Result<()> {
  Config::set_log_level()?;

  let config = Config::parse()?;
  let keys = Keys::new_public(&PUBLIC_KEY).context("public key should be ECDSA in PEM format")?;

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

  match config.api {
    true => {
      if let Err(err) = run_api(pool, config, keys).await {
        log_error(&err);
      }
    }

    false => future::pending().await,
  }

  Ok(())
}

async fn run_api(pool: Pool<MySql>, config: Arc<Config>, keys: Keys<'static>) -> Result<()> {
  log::info!("started API process on port {}", config.api_port);

  let mut provider = RocketConfig::release_default();
  provider.port = config.api_port;

  api::server(provider, pool, keys).attach(ApiLogger::new()).launch().await.context("could not launch API handler")?;

  Ok(())
}

async fn run_defcon(pool: &Pool<MySql>, config: Arc<Config>) {
  log::info!("started handler process");

  let inhibitor = Inhibitor::new();

  loop {
    let next_tick_at = Instant::now() + config.handler_interval;

    tokio::spawn({
      let pool = pool.clone();
      let config = config.clone();
      let inhibitor = inhibitor.clone();

      async move {
        if let Err(err) = handler::tick(pool, config, inhibitor).await {
          log_error(&err);
        }
      }
    });

    tokio::time::delay_until(next_tick_at.into()).await;
  }
}

async fn run_cleaner(pool: &Pool<MySql>, config: Arc<Config>) {
  log::info!("started cleaner process");

  loop {
    let config = config.clone();
    let next_tick_at = Instant::now() + config.cleaner_interval;

    tokio::spawn({
      let pool = pool.clone();

      async move {
        cleaner::tick(pool, config).await;
      }
    });

    tokio::time::delay_until(next_tick_at.into()).await;
  }
}
