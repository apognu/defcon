use anyhow::Context;
use axum::{
  extract::{Path, State},
  Json,
};
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

pub async fn get(_: Auth, pool: State<Pool<MySql>>, Path(uuid): Path<String>) -> ApiResponse<Json<Vec<api::Timeline>>> {
  let pool = &pool;
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let outage = db::Outage::by_uuid(&mut conn, &uuid).await.context("could not retrieve outage").short()?;

  let timeline = db::Timeline::for_outage(&mut conn, &outage)
    .await
    .context("could not retrieve timeline")
    .short()?
    .map(pool)
    .await
    .short()?;

  Ok(Json(timeline))
}
