use anyhow::Context;
use pulldown_cmark::{html, Parser};
use rocket::{
  serde::json::{Error as JsonError, Json},
  State,
};
use sqlx::{MySql, Pool};

use crate::{
  api::{
    auth::Auth,
    error::{check_json, Shortable},
    types::{self as api, ApiMapper},
    ApiResponse,
  },
  model as db,
};

#[get("/api/outages", rank = 10)]
pub async fn list(_auth: Auth, pool: &State<Pool<MySql>>) -> ApiResponse<Json<Vec<api::Outage>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let outages = db::Outage::current(&mut conn).await.context("could not retrieve outages").short()?.map(pool).await.short()?;

  Ok(Json(outages))
}

#[get("/api/outages?<check>&<from>&<to>&<limit>&<page>", rank = 5)]
pub async fn list_between(_auth: Auth, pool: &State<Pool<MySql>>, check: Option<String>, from: api::Date, to: api::Date, limit: Option<u8>, page: Option<u8>) -> ApiResponse<Json<Vec<api::Outage>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let check = match check {
    Some(uuid) => Some(db::Check::by_uuid(&mut conn, &uuid).await.context("could not retrieve check").short()?),
    None => None,
  };

  let outages = db::Outage::between(&mut conn, check.as_ref(), from.and_hms(0, 0, 0), to.and_hms(23, 59, 59), limit, page)
    .await
    .context("could not retrieve outages")
    .short()?
    .map(pool)
    .await
    .short()?;

  Ok(Json(outages))
}

#[get("/api/outages/<uuid>")]
pub async fn get(_auth: Auth, pool: &State<Pool<MySql>>, uuid: String) -> ApiResponse<Json<api::Outage>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let outage = db::Outage::by_uuid(&mut conn, &uuid).await.context("could not retrieve outage").short()?.map(pool).await.short()?;

  Ok(Json(outage))
}

#[get("/api/checks/<uuid>/outages?<limit>&<page>", rank = 10)]
pub async fn list_for_check(_auth: Auth, pool: &State<Pool<MySql>>, uuid: String, limit: Option<u8>, page: Option<u8>) -> ApiResponse<Json<Vec<api::Outage>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let check = db::Check::by_uuid(&mut conn, &uuid).await.context("could not retrieve check").short()?;

  let outages = db::Outage::for_check(&mut conn, &check, limit, page)
    .await
    .context("could not retrieve outages")
    .short()?
    .map(pool)
    .await
    .short()?;

  Ok(Json(outages))
}

#[get("/api/checks/<uuid>/outages?<from>&<to>&<limit>&<page>", rank = 5)]
pub async fn list_for_check_between(
  _auth: Auth,
  pool: &State<Pool<MySql>>,
  uuid: String,
  from: api::DateTime,
  to: api::DateTime,
  limit: Option<u8>,
  page: Option<u8>,
) -> ApiResponse<Json<Vec<api::Outage>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let check = db::Check::by_uuid(&mut conn, &uuid).await.context("could not retrieve check").short()?;

  let outages = db::Outage::for_check_between(&mut conn, &check, *from, *to, limit, page)
    .await
    .context("could not retrieve outages")
    .short()?
    .map(pool)
    .await
    .short()?;

  Ok(Json(outages))
}

#[put("/api/outages/<uuid>/comment", data = "<payload>")]
pub async fn comment(auth: Auth, pool: &State<Pool<MySql>>, uuid: String, payload: Result<Json<api::OutageComment>, JsonError<'_>>) -> ApiResponse<()> {
  let payload = check_json(payload).short()?;
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let outage = db::Outage::by_uuid(&mut conn, &uuid).await.context("could not retrieve outage").short()?;

  let mut html_comment = String::new();

  {
    html::push_html(&mut html_comment, Parser::new(&payload.comment));
  }

  db::Timeline::new(outage.id, Some(auth.user.id), "comment", &html_comment)
    .insert(&mut conn)
    .await
    .context("could not add comment")
    .short()?;

  Ok(())
}
