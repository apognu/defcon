use std::sync::Arc;

use anyhow::{Context, Result};
use rocket::{get, http::Status, response::status::Custom, routes, Config as RocketConfig, Route, State};
use sqlx::{MySql, Pool};

use defcon::{
  config::Config,
  model::{Check, DeadManSwitchLog},
};

pub async fn run(pool: Pool<MySql>, config: Arc<Config>) {
  let provider = RocketConfig {
    address: config.dms_listen.ip(),
    port: config.dms_listen.port(),
    ..RocketConfig::release_default()
  };

  rocket::custom(provider).manage(pool).mount("/", routes()).launch().await.unwrap();
}

fn routes() -> Vec<Route> {
  routes![checkin]
}

#[get("/checkin/<uuid>")]
async fn checkin(pool: &State<Pool<MySql>>, uuid: String) -> Result<(), Custom<()>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").map_err(|_| Custom(Status::NotFound, ()))?;
  let check = Check::by_uuid(&mut conn, &uuid).await.map_err(|_| Custom(Status::NotFound, ()))?;

  DeadManSwitchLog::insert(&mut conn, check.id).await.map_err(|_| Custom(Status::InternalServerError, ()))?;

  Ok(())
}
