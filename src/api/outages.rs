use anyhow::Context;
use axum::{
  extract::{rejection::JsonRejection, Path, Query, State},
  response::IntoResponse,
  Json,
};
use pulldown_cmark::{html, Parser};
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

use super::error::AppError;

#[derive(Deserialize)]
pub struct ListQuery {
  check: Option<String>,
  from: Option<api::Date>,
  to: Option<api::Date>,
  limit: Option<u8>,
  page: Option<u8>,
}

pub async fn list(_: Auth, ref pool: State<Pool<MySql>>, Query(ListQuery { check, from, to, limit, page }): Query<ListQuery>) -> ApiResponse<Json<Vec<api::Outage>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let outages = if let Some((from, to)) = from.zip(to) {
    let check = match check {
      Some(uuid) => Some(db::Check::by_uuid(&mut conn, &uuid).await.context("could not retrieve check").short()?),
      None => None,
    };

    db::Outage::between(&mut conn, check.as_ref(), from.and_hms(0, 0, 0), to.and_hms(23, 59, 59), limit, page)
      .await
      .context("could not retrieve outages")
      .short()?
      .map(pool)
      .await
      .short()?
  } else {
    db::Outage::current(&mut conn).await.context("could not retrieve outages").short()?.map(pool).await.short()?
  };

  Ok(Json(outages))
}

pub async fn get(_: Auth, ref pool: State<Pool<MySql>>, Path(uuid): Path<String>) -> ApiResponse<Json<api::Outage>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let outage = db::Outage::by_uuid(&mut conn, &uuid).await.context("could not retrieve outage").short()?.map(pool).await.short()?;

  Ok(Json(outage))
}

#[derive(Deserialize)]
pub struct ListForOutageQuery {
  from: Option<api::DateTime>,
  to: Option<api::DateTime>,
  limit: Option<u8>,
  page: Option<u8>,
}

pub async fn list_for_check(
  _: Auth,
  ref pool: State<Pool<MySql>>,
  Path(uuid): Path<String>,
  Query(ListForOutageQuery { from, to, limit, page }): Query<ListForOutageQuery>,
) -> ApiResponse<Json<Vec<api::Outage>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let check = db::Check::by_uuid(&mut conn, &uuid).await.context("could not retrieve check").short()?;

  let outages = if let Some((from, to)) = from.zip(to) {
    db::Outage::for_check_between(&mut conn, &check, *from, *to, limit, page)
      .await
      .context("could not retrieve outages")
      .short()?
      .map(pool)
      .await
      .short()?
  } else {
    db::Outage::for_check(&mut conn, &check, limit, page)
      .await
      .context("could not retrieve outages")
      .short()?
      .map(pool)
      .await
      .short()?
  };

  Ok(Json(outages))
}

pub async fn acknowledge(auth: Auth, pool: State<Pool<MySql>>, Path(uuid): Path<String>) -> ApiResponse<impl IntoResponse> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let outage = db::Outage::by_uuid(&mut conn, &uuid).await.context("could not retrieve outage").short()?;

  if outage.acknowledged_by.is_some() {
    Err(anyhow!("outage was already acknowledged").context(AppError::Conflict)).short()?;
  }

  outage.acknowledge(&mut conn, &auth.user).await.short()?;

  db::Timeline::new(outage.id, Some(auth.user.id), "acknowledgement", "")
    .insert(&mut conn)
    .await
    .context("could not add comment")
    .short()?;

  Ok(())
}

pub async fn comment(auth: Auth, pool: State<Pool<MySql>>, Path(uuid): Path<String>, payload: Result<Json<api::OutageComment>, JsonRejection>) -> ApiResponse<()> {
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
