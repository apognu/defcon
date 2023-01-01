mod alerters;
pub mod auth;
mod checks;
pub mod error;
mod events;
mod groups;
pub mod middlewares;
mod outages;
mod runner;
mod session;
mod site_outages;
mod status;
mod timeline;
pub mod types;
mod users;

use std::sync::Arc;

use axum::{
  extract::{FromRef, State},
  http::StatusCode,
  middleware,
  response::{ErrorResponse, IntoResponse, Json},
  routing::{get, post, put},
  Router,
};
use sqlx::{MySql, Pool};

use crate::{
  api::{auth::Keys, middlewares::api_logger},
  config::Config,
};

use self::{
  auth::Auth,
  error::{AppError, Shortable},
};

#[derive(Clone, FromRef)]
pub struct AppState {
  pub config: Arc<Config>,
  pub pool: Pool<MySql>,
  pub keys: Option<Keys>,
}

type ApiResponse<T> = Result<T, ErrorResponse>;

pub fn server(config: Arc<Config>, pool: Pool<MySql>, keys: Option<Keys>) -> Router {
  let state = AppState { config, pool, keys };

  let router = Router::new();
  let router = api_router(router, state.clone());
  let router = web_router(router, state.clone());
  let router = runner_routes(router, state);

  router.layer(middleware::from_fn(api_logger))
}

pub fn api_router(router: Router, state: AppState) -> Router {
  let routes = Router::new()
    .route("/-/health", get(health))
    .route("/-/config", get(configuration))
    .route("/-/token", post(session::token))
    .route("/-/refresh", post(session::refresh))
    .route("/-/me", get(session::userinfo))
    .route("/-/password", post(session::password))
    .route("/-/apikey", post(session::api_key))
    .route("/checks", get(checks::list).post(checks::create))
    .route("/checks/:uuid", get(checks::get).put(checks::update).patch(checks::patch).delete(checks::delete))
    .route("/checks/:uuid/outages", get(outages::list_for_check))
    .route("/checks/:uuid/events", get(events::list_for_check))
    .route("/groups", get(groups::list).post(groups::create))
    .route("/groups/:uuid", get(groups::get).put(groups::update).delete(groups::delete))
    .route("/sites/outages", get(site_outages::list))
    .route("/sites/outages/:uuid", get(site_outages::get))
    .route("/outages", get(outages::list))
    .route("/outages/:uuid", get(outages::get))
    .route("/outages/:uuid/acknowledge", post(outages::acknowledge))
    .route("/outages/:uuid/comment", put(outages::comment))
    .route("/outages/:uuid/events", get(events::list_for_outage))
    .route("/alerters", get(alerters::list).post(alerters::add))
    .route("/alerters/:uuid", get(alerters::get).put(alerters::update).delete(alerters::delete))
    .route("/status", get(status::status))
    .route("/statistics", get(status::statistics))
    .route("/outages/:uuid/timeline", get(timeline::get))
    .route("/users", get(users::list).post(users::create))
    .route("/users/:uuid", get(users::get).put(users::update).patch(users::patch).delete(users::delete));

  #[cfg(feature = "web")]
  let routes = if state.config.web.enable_status_page {
    routes.route("/status-page", get(status::status_page))
  } else {
    routes
  };

  let routes = routes.fallback(api_catchall).with_state(state);

  router.nest("/api", routes)
}

pub fn runner_routes(router: Router, state: AppState) -> Router {
  match state.keys {
    Some(_) => router.nest(
      "/runner",
      Router::new().route("/checks", get(runner::list_stale)).route("/report", post(runner::report)).with_state(state),
    ),

    None => {
      log::info!("no public key found, disabling runner endpoints");

      router
    }
  }
}

#[cfg(feature = "web")]
pub fn web_router(router: Router, state: AppState) -> Router {
  if state.config.web.enable {
    return crate::web::router(router);
  }

  router
}

#[cfg(not(feature = "web"))]
pub fn web_router(router: Router, _: AppState) -> Router {
  router
}

async fn health() -> StatusCode {
  StatusCode::OK
}

async fn configuration(_: Auth, State(config): State<Arc<Config>>) -> Json<Arc<Config>> {
  Json(config)
}

async fn api_catchall() -> impl IntoResponse {
  // StatusCode::NOT_FOUND

  Err::<(), _>(anyhow!(AppError::ResourceNotFound)).short()
}

#[cfg(test)]
mod tests {
  use axum::{
    body::Body,
    http::{Request, StatusCode},
  };
  use tower::ServiceExt;

  use crate::tests;

  #[tokio::test]
  async fn health() {
    let (pool, client) = tests::api_client().await.unwrap();

    let response = client.oneshot(Request::builder().uri("/api/-/health").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    pool.cleanup().await;
  }
}
