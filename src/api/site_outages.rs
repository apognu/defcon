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
  model::SiteOutage,
};

#[get("/api/sites/outages", rank = 10)]
pub async fn list(_auth: Auth, pool: &State<Pool<MySql>>) -> ApiResponse<Json<Vec<api::SiteOutage>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let outages = SiteOutage::current(&mut conn).await.context("could not retrieve outages").short()?.map(pool).await.short()?;

  Ok(Json(outages))
}

#[get("/api/sites/outages?<from>&<to>", rank = 5)]
pub async fn list_between(_auth: Auth, pool: &State<Pool<MySql>>, from: api::DateTime, to: api::DateTime) -> ApiResponse<Json<Vec<api::SiteOutage>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let outages = SiteOutage::between(&mut conn, *from, *to)
    .await
    .context("could not retrieve outages")
    .short()?
    .map(pool)
    .await
    .short()?;

  Ok(Json(outages))
}

#[get("/api/sites/outages/<uuid>")]
pub async fn get(_auth: Auth, pool: &State<Pool<MySql>>, uuid: String) -> ApiResponse<Json<api::SiteOutage>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let outage = SiteOutage::by_uuid(&mut conn, &uuid).await.context("could not find outage").short()?.map(pool).await.short()?;

  Ok(Json(outage))
}

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use rocket::http::Status;
  use uuid::Uuid;

  use crate::{model::SiteOutage, tests};

  #[tokio::test]
  async fn list() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(None, None, "list()", None, None).await?;
    pool.create_unresolved_site_outage(Some(1), None).await?;
    pool.create_resolved_site_outage(Some(2), Some(Uuid::new_v4().to_string())).await?;

    let response = client.get("/api/sites/outages").dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    let outages: Vec<SiteOutage> = serde_json::from_str(&response.into_string().await.unwrap())?;
    assert_eq!(outages.len(), 1);
    assert_eq!(&outages[0].uuid, "dd9a531a-1b0b-4a12-bc09-e5637f916261");

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn list_between() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(None, None, "list_between()", None, None).await?;
    pool.create_unresolved_site_outage(Some(1), Some(Uuid::new_v4().to_string())).await?;
    pool.create_resolved_site_outage(Some(2), Some(Uuid::new_v4().to_string())).await?;

    let response = client.get("/api/sites/outages?from=2020-12-31T00:00:00&to=2021-12-31T00:00:00").dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    let outages: Vec<SiteOutage> = serde_json::from_str(&response.into_string().await.unwrap())?;
    assert_eq!(outages.len(), 2);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn get() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(None, None, "get()", None, None).await?;
    pool.create_unresolved_site_outage(Some(1), None).await?;

    let response = client.get("/api/sites/outages/dd9a531a-1b0b-4a12-bc09-e5637f916261").dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    let outages: SiteOutage = serde_json::from_str(&response.into_string().await.unwrap())?;
    assert_eq!(&outages.uuid, "dd9a531a-1b0b-4a12-bc09-e5637f916261");

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn get_not_found() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(None, None, "get_not_found()", None, None).await?;
    pool.create_unresolved_site_outage(Some(1), None).await?;

    let response = client.get("/api/sites/outages/nonexistant").dispatch().await;
    assert_eq!(response.status(), Status::NotFound);

    pool.cleanup().await;

    Ok(())
  }
}
