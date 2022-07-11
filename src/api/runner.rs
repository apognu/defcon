use std::sync::Arc;

use anyhow::Context;
use rocket::{serde::json::Json, State};
use sqlx::{MySql, Pool};

use crate::{
  api::{
    auth::RunnerAuth,
    error::Shortable,
    types::{self as api, ApiMapper},
    ApiResponse,
  },
  config::Config,
  handlers,
  model::{Check, Event},
};

#[get("/api/runner/checks")]
pub async fn list_stale(pool: &State<Pool<MySql>>, credentials: RunnerAuth) -> ApiResponse<Json<Vec<api::RunnerCheck>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let checks: Vec<api::RunnerCheck> = Check::stale(&mut conn, &credentials.site)
    .await
    .context("could not retrieve checks")
    .short()?
    .map(pool)
    .await
    .short()?
    .into_iter()
    .map(Into::into)
    .collect();

  Ok(Json(checks))
}

#[post("/api/runner/report", data = "<payload>")]
pub async fn report(config: &State<Arc<Config>>, pool: &State<Pool<MySql>>, credentials: RunnerAuth, payload: Json<api::ReportEvent>) -> ApiResponse<()> {
  let report = payload.0;
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let check = Check::by_uuid(&mut conn, &report.check).await.short()?;

  let event = Event {
    check_id: check.id,
    site: credentials.site.clone(),
    status: report.status,
    message: report.message,
    ..Default::default()
  };

  handlers::handle_event(config.inner().clone(), &mut conn, &event, &check, None).await.short()?;

  Ok(())
}
