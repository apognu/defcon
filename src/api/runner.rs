use anyhow::Context;
use rocket::State;
use rocket_contrib::json::Json;
use sqlx::{MySql, Pool};

use crate::{
  api::{
    auth::RunnerCredentials,
    error::Shortable,
    types::{self as api, ApiMapper},
    ApiResponse,
  },
  handlers,
  model::{Check, Event},
};

#[get("/api/runner/checks")]
pub async fn list_stale(pool: State<'_, Pool<MySql>>, credentials: RunnerCredentials) -> ApiResponse<Json<Vec<api::RunnerCheck>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let checks: Vec<api::RunnerCheck> = Check::stale(&mut conn, &credentials.site)
    .await
    .context("could not retrieve checks")
    .short()?
    .map(&*pool)
    .await
    .short()?
    .into_iter()
    .map(Into::into)
    .collect();

  Ok(Json(checks))
}

#[post("/api/runner/report", data = "<payload>")]
pub async fn report(pool: State<'_, Pool<MySql>>, credentials: RunnerCredentials, payload: Json<api::ReportEvent>) -> ApiResponse<()> {
  let report = payload.0;
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let check = Check::by_uuid(&mut *conn, &report.check).await.short()?;

  let event = Event {
    check_id: check.id,
    site: credentials.site.clone(),
    status: report.status,
    message: report.message,
    ..Default::default()
  };

  handlers::handle_event(&mut *conn, &credentials.site, &event, &check, None).await.short()?;

  Ok(())
}
