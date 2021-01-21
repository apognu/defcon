use anyhow::Context;
use rocket::State;
use rocket_contrib::json::{Json, JsonError};
use sqlx::{MySql, Pool};

use crate::{
  api::{
    error::{check_json, Errorable},
    types::{self as api, ApiMapper},
    ApiResponse,
  },
  model::Outage,
};

#[get("/api/outages")]
pub async fn list(pool: State<'_, Pool<MySql>>) -> ApiResponse<Json<Vec<api::Outage>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").apierr()?;
  let outages = Outage::current(&mut conn).await.apierr()?.map(&*pool).await.apierr()?;

  Ok(Json(outages))
}

#[get("/api/outages?<from>&<end>")]
pub async fn list_between(pool: State<'_, Pool<MySql>>, from: api::DateTime, end: api::DateTime) -> ApiResponse<Json<Vec<api::Outage>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").apierr()?;
  let outages = Outage::between(&mut conn, *from, *end).await.apierr()?.map(&*pool).await.apierr()?;

  Ok(Json(outages))
}

#[get("/api/outages/<uuid>")]
pub async fn get(pool: State<'_, Pool<MySql>>, uuid: String) -> ApiResponse<Json<api::Outage>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").apierr()?;
  let outage = Outage::by_uuid(&mut conn, &uuid).await.apierr()?.map(&*pool).await.apierr()?;

  Ok(Json(outage))
}

#[put("/api/outages/<uuid>/comment", data = "<payload>")]
pub async fn comment(pool: State<'_, Pool<MySql>>, uuid: String, payload: Result<Json<api::OutageComment>, JsonError<'_>>) -> ApiResponse<()> {
  let payload = check_json(payload).apierr()?;
  let mut conn = pool.acquire().await.context("could not retrieve database connection").apierr()?;
  let outage = Outage::by_uuid(&mut conn, &uuid).await.apierr()?;

  outage.comment(&mut conn, &payload.comment).await.apierr()?;

  Ok(())
}
