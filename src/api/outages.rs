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

#[get("/api/outages", rank = 10)]
pub async fn list(pool: State<'_, Pool<MySql>>) -> ApiResponse<Json<Vec<api::Outage>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").apierr()?;
  let outages = Outage::current(&mut conn).await.apierr()?.map(&*pool).await.apierr()?;

  Ok(Json(outages))
}

#[get("/api/outages?<start>&<end>", rank = 5)]
pub async fn list_between(pool: State<'_, Pool<MySql>>, start: api::DateTime, end: api::DateTime) -> ApiResponse<Json<Vec<api::Outage>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").apierr()?;
  let outages = Outage::between(&mut conn, *start, *end).await.apierr()?.map(&*pool).await.apierr()?;

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

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use rocket::http::Status;
  use rocket_contrib::json;
  use uuid::Uuid;

  use crate::{model::Outage, spec};

  #[tokio::test]
  async fn list() -> Result<()> {
    let (pool, client) = spec::api_client().await?;

    pool.create_check(None, None, "list()", None).await?;
    pool.create_unresolved_outage(Some(1), None).await?;
    pool.create_resolved_outage(Some(2), Some(Uuid::new_v4().to_string())).await?;

    let response = client.get("/api/outages").dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    let outages: Vec<Outage> = serde_json::from_str(&response.into_string().await.unwrap())?;
    assert_eq!(outages.len(), 1);
    assert_eq!(&outages[0].uuid, "dd9a531a-1b0b-4a12-bc09-e5637f916261");

    Ok(())
  }

  #[tokio::test]
  async fn list_between() -> Result<()> {
    let (pool, client) = spec::api_client().await?;

    pool.create_check(None, None, "list_between()", None).await?;
    pool.create_unresolved_outage(Some(1), Some(Uuid::new_v4().to_string())).await?;
    pool.create_resolved_outage(Some(2), Some(Uuid::new_v4().to_string())).await?;

    let response = client.get("/api/outages?start=2020-12-31T00:00:00&end=2021-12-31T00:00:00").dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    let outages: Vec<Outage> = serde_json::from_str(&response.into_string().await.unwrap())?;
    assert_eq!(outages.len(), 2);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn get() -> Result<()> {
    let (pool, client) = spec::api_client().await?;

    pool.create_check(None, None, "get()", None).await?;
    pool.create_unresolved_outage(Some(1), None).await?;

    let response = client.get("/api/outages/dd9a531a-1b0b-4a12-bc09-e5637f916261").dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    let outages: Outage = serde_json::from_str(&response.into_string().await.unwrap())?;
    assert_eq!(&outages.uuid, "dd9a531a-1b0b-4a12-bc09-e5637f916261");

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn get_not_found() -> Result<()> {
    let (pool, client) = spec::api_client().await?;

    pool.create_check(None, None, "get_not_found()", None).await?;
    pool.create_unresolved_outage(Some(1), None).await?;

    let response = client.get("/api/outages/nonexistant").dispatch().await;
    assert_eq!(response.status(), Status::NotFound);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn comment() -> Result<()> {
    let (pool, client) = spec::api_client().await?;

    pool.create_check(None, None, "comment()", None).await?;
    pool.create_unresolved_outage(Some(1), None).await?;

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
