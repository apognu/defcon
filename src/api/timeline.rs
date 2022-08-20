use anyhow::Context;
use rocket::{serde::json::Json, State};
use sqlx::{MySql, Pool};

use crate::{
  api::{
    auth::Auth,
    error::Shortable,
    types::{self as api, ApiMapper},
    ApiResponse,
  },
  model as db,
};

#[get("/api/outages/<uuid>/timeline")]
pub async fn get(_auth: Auth, pool: &State<Pool<MySql>>, uuid: &str) -> ApiResponse<Json<Vec<api::Timeline>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let outage = db::Outage::by_uuid(&mut conn, uuid).await.context("could not retrieve outage").short()?;

  let timeline = db::Timeline::for_outage(&mut conn, &outage)
    .await
    .context("could not retrieve timeline")
    .short()?
    .map(pool)
    .await
    .short()?;

  Ok(Json(timeline))
}
