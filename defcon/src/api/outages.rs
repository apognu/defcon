use anyhow::Context;
use rocket::{
  serde::json::{Error as JsonError, Json},
  State,
};
use sqlx::{MySql, Pool};

use crate::{
  api::{
    error::{check_json, Shortable},
    types::{self as api, ApiMapper},
    ApiResponse,
  },
  model::Outage,
};

#[get("/api/outages", rank = 10)]
pub async fn list(pool: &State<Pool<MySql>>) -> ApiResponse<Json<Vec<api::Outage>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let outages = Outage::current(&mut conn).await.context("could not retrieve outages").short()?.map(&*pool).await.short()?;

  Ok(Json(outages))
}

#[get("/api/outages?<from>&<to>", rank = 5)]
pub async fn list_between(pool: &State<Pool<MySql>>, from: api::DateTime, to: api::DateTime) -> ApiResponse<Json<Vec<api::Outage>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let outages = Outage::between(&mut conn, *from, *to).await.context("could not retrieve outages").short()?.map(&*pool).await.short()?;

  Ok(Json(outages))
}

#[put("/api/outages/<uuid>/comment", data = "<payload>")]
pub async fn comment(pool: &State<Pool<MySql>>, uuid: String, payload: Result<Json<api::OutageComment>, JsonError<'_>>) -> ApiResponse<()> {
  let payload = check_json(payload).short()?;
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let outage = Outage::by_uuid(&mut conn, &uuid).await.context("could not retrieve outage").short()?;

  outage.comment(&mut conn, &payload.comment).await.context("could not add comment to outage").short()?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use rocket::{http::Status, serde::json::json};

  use crate::tests;

  #[tokio::test]
  async fn comment() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(None, None, "comment()", None, None).await?;
    pool.create_resolved_outage(None, None).await?;

    let payload = json!({
      "comment": "lorem ipsum"
    });

    let response = client
      .put("/api/outages/dd9a531a-1b0b-4a12-bc09-e5637f916261/comment")
      .body(payload.to_string().as_bytes())
      .dispatch()
      .await;

    assert_eq!(response.status(), Status::Ok);

    let outage = sqlx::query_as::<_, (String,)>(r#"SELECT comment FROM outages WHERE uuid = "dd9a531a-1b0b-4a12-bc09-e5637f916261""#)
      .fetch_one(&*pool)
      .await?;

    assert_eq!(&outage.0, "lorem ipsum");

    pool.cleanup().await;

    Ok(())
  }
}
