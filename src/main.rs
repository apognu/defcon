#![feature(async_closure, try_find)]
#![deny(clippy::all)]
#![allow(clippy::unit_arg)]

#[macro_use]
extern crate serde;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate anyhow;

mod alerters;
mod api;
mod cleaner;
mod config;
mod defcon;
mod ext;
mod handlers;
mod inhibitor;
mod model;

use std::{env, sync::Arc, time::Instant};

use anyhow::{Context, Error, Result};
use futures::future;
use kvlogger::KvLoggerBuilder;
use rocket::config::Config as RocketConfig;
use sqlx::{
  mysql::{MySql, MySqlPoolOptions},
  Pool,
};

use crate::{api::middlewares::ApiLogger, config::Config, inhibitor::Inhibitor, model::migrations};

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
async fn main() {
  if let Err(err) = start().await {
    log_error(&err);
  }
}

async fn start() -> Result<()> {
  Config::set_log_level();
  KvLoggerBuilder::default().init()?;

  let config = Config::parse()?;

  let dsn = env::var("DSN")?;
  let pool = MySqlPoolOptions::new().max_connections(20).connect(&dsn).await?;

  migrations::migrate()?;

  if !config.handler && !config.cleaner && !config.api {
    return Err(anyhow!("all processes disabled, aborting"));
  }

  tokio::spawn({
    let config = config.clone();
    let pool = pool.clone();

    async move {
      if config.handler {
        run_defcon(&pool, config).await;
      }
    }
  });

  tokio::spawn({
    let config = config.clone();
    let pool = pool.clone();

    async move {
      if config.cleaner {
        run_cleaner(&pool, config).await;
      }
    }
  });

  match config.api {
    true => {
      if let Err(err) = run_api(pool, config).await {
        log_error(&err);
      }
    }

    false => future::pending().await,
  }

  Ok(())
}

async fn run_api(pool: Pool<MySql>, config: Arc<Config>) -> Result<()> {
  log::info!("started API process on port {}", config.api_port);

  let mut provider = RocketConfig::release_default();
  provider.port = config.api_port;

  rocket::custom(provider)
    .manage(pool)
    .attach(ApiLogger::new())
    .mount("/", api::routes())
    .register(catchers![api::not_found, api::unprocessable])
    .launch()
    .await
    .context("could not launch API handler")?;

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
        if let Err(err) = defcon::tick(pool, config, inhibitor).await {
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
