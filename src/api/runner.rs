use anyhow::Context;
use rocket::State;
use rocket_contrib::json::Json;
use sqlx::{MySql, Pool};

use crate::{
  api::{
    error::Errorable,
    types::{self as api, ApiMapper},
    ApiResponse,
  },
  handlers,
  model::{Check, Event},
};

#[get("/api/checks/stale?<site>")]
pub async fn list_stale(pool: State<'_, Pool<MySql>>, site: String) -> ApiResponse<Json<Vec<api::Check>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").apierr()?;
  let checks = Check::stale(&mut conn, &site).await.apierr()?.map(&*pool).await.apierr()?;

  Ok(Json(checks))
}

#[post("/api/checks/report?<site>", data = "<payload>")]
pub async fn report(pool: State<'_, Pool<MySql>>, site: String, payload: Json<api::ReportEvent>) {
  let report = payload.0;
  let mut conn = pool.acquire().await.unwrap();

  let check = Check::by_uuid(&mut *conn, &report.check).await.unwrap();

  let event = Event {
    check_id: check.id,
    status: report.status,
    message: report.message,
    ..Default::default()
  };

  handlers::handle_event(&mut *conn, &site, &event, &check, None).await.unwrap();
}
