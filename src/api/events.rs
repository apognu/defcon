use anyhow::Context;
use rocket::State;
use rocket_contrib::json::Json;
use sqlx::{MySql, Pool};

use crate::{
  api::{error::Errorable, ApiResponse},
  model as db,
};

#[get("/api/outages/<uuid>/events")]
pub async fn list(pool: State<'_, Pool<MySql>>, uuid: String) -> ApiResponse<Json<Vec<db::Event>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").apierr()?;
  let outage = db::Outage::by_uuid(&mut conn, &uuid).await.apierr()?;
  let events = db::Event::for_outage(&mut conn, &outage).await.apierr()?;

  Ok(Json(events))
}
