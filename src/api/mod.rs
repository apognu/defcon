mod alerters;
pub mod auth;
mod checks;
pub mod error;
mod events;
mod groups;
pub mod middlewares;
mod outages;
mod runner;
mod site_outages;
mod status;
pub mod types;

use std::sync::Arc;

use rocket::{
  serde::json::{json, Value as JsonValue},
  Build, Config as RocketConfig, Rocket, Route,
};
use sqlx::{MySql, Pool};

use crate::{
  api::{auth::Keys, error::ErrorResponse},
  config::Config,
};

type ApiResponse<T> = Result<T, ErrorResponse>;

pub fn server(provider: RocketConfig, config: Arc<Config>, pool: Pool<MySql>, keys: Option<Keys<'static>>) -> Rocket<Build> {
  let routes: Vec<Route> = routes().into_iter().chain(runner_routes(&keys).into_iter()).collect();

  match keys {
    Some(keys) => rocket::custom(provider)
      .manage(config)
      .manage(pool)
      .manage(keys)
      .mount("/", routes)
      .register("/", catchers![not_found, unprocessable]),
    None => rocket::custom(provider)
      .manage(config)
      .manage(pool)
      .mount("/", routes)
      .register("/", catchers![not_found, unprocessable]),
  }
}

pub fn routes() -> Vec<Route> {
  routes![
    health,
    checks::list,
    checks::get,
    checks::create,
    checks::update,
    checks::patch,
    checks::delete,
    groups::list,
    groups::get,
    groups::create,
    groups::update,
    groups::delete,
    site_outages::list,
    site_outages::list_between,
    site_outages::get,
    outages::list,
    outages::list_between,
    outages::list_for_check,
    outages::list_for_check_between,
    outages::comment,
    events::list_for_check,
    events::list_for_check_between,
    events::list_for_outage,
    alerters::list,
    alerters::get,
    alerters::add,
    alerters::update,
    status::status,
    status::statistics,
  ]
}

pub fn runner_routes(keys: &Option<Keys<'static>>) -> Vec<Route> {
  match keys {
    Some(_) => routes![runner::list_stale, runner::report,],

    None => {
      log::info!("no public key found, disabling runner endpoints");

      vec![]
    }
  }
}

#[get("/api/-/health")]
fn health() {}

#[catch(404)]
pub fn not_found() -> JsonValue {
  json!({
    "status": "not_found",
    "message": "requested resource was not found"
  })
}

#[catch(422)]
pub fn unprocessable() -> JsonValue {
  json!({
    "status": "unprocessable",
    "message": "the data you provided could not be understood"
  })
}

#[cfg(test)]
mod tests {
  use rocket::http::Status;

  use crate::tests;

  #[tokio::test]
  async fn health() {
    let (pool, client) = tests::api_client().await.unwrap();

    let response = client.get("/api/-/health").dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    pool.cleanup().await;
  }
}
