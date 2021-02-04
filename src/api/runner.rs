use anyhow::Context;
use rocket::State;
use rocket_contrib::json::Json;
use sqlx::{MySql, Pool};

use crate::{
  api::{
    error::Shortable,
    types::{self as api, ApiMapper},
    ApiResponse,
  },
  handlers,
  model::{Check, Event},
};

#[get("/api/checks/stale?<site>")]
pub async fn list_stale(pool: State<'_, Pool<MySql>>, site: String) -> ApiResponse<Json<Vec<api::RunnerCheck>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let checks: Vec<api::RunnerCheck> = Check::stale(&mut conn, &site)
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

#[post("/api/checks/report?<site>", data = "<payload>")]
pub async fn report(pool: State<'_, Pool<MySql>>, site: String, payload: Json<api::ReportEvent>) -> ApiResponse<()> {
  let report = payload.0;
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let check = Check::by_uuid(&mut *conn, &report.check).await.short()?;

  let event = Event {
    check_id: check.id,
    site: site.clone(),
    status: report.status,
    message: report.message,
    ..Default::default()
  };

  handlers::handle_event(&mut *conn, &site, &event, &check, None).await.short()?;

  Ok(())
}
