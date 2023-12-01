use anyhow::Context;
use axum::{
  extract::{Path, Query, State},
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
  model::SiteOutage,
};

#[derive(Deserialize)]
pub struct ListQuery {
  from: Option<api::DateTime>,
  to: Option<api::DateTime>,
}

pub async fn list(_: Auth, pool: State<Pool<MySql>>, Query(ListQuery { from, to }): Query<ListQuery>) -> ApiResponse<Json<Vec<api::SiteOutage>>> {
  let pool = &pool;
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let outages = if let Some((from, to)) = from.zip(to) {
    SiteOutage::between(&mut conn, *from, *to)
      .await
      .context("could not retrieve outages")
      .short()?
      .map(pool)
      .await
      .short()?
  } else {
    SiteOutage::current(&mut conn).await.context("could not retrieve outages").short()?.map(pool).await.short()?
  };

  Ok(Json(outages))
}

pub async fn get(_: Auth, pool: State<Pool<MySql>>, Path(uuid): Path<String>) -> ApiResponse<Json<api::SiteOutage>> {
  let pool = &pool;
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let outage = SiteOutage::by_uuid(&mut conn, &uuid).await.context("could not find outage").short()?.map(pool).await.short()?;

  Ok(Json(outage))
}

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use axum::{
    body::Body,
    http::{Request, StatusCode},
  };
  use http_body_util::BodyExt;
  use tower::ServiceExt;
  use uuid::Uuid;

  use crate::{model::SiteOutage, tests};

  #[tokio::test]
  async fn list() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(None, None, "list()", None, None).await?;
    pool.create_unresolved_site_outage(Some(1), None).await?;
    pool.create_resolved_site_outage(Some(2), Some(Uuid::new_v4().to_string())).await?;

    let response = client.oneshot(Request::builder().uri("/api/sites/outages").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let outages: Vec<SiteOutage> = serde_json::from_slice(response.into_body().collect().await.unwrap().to_bytes().as_ref())?;
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

    let response = client
      .oneshot(
        Request::builder()
          .uri("/api/sites/outages?from=2020-12-31T00:00:00&to=2021-12-31T00:00:00")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let outages: Vec<SiteOutage> = serde_json::from_slice(response.into_body().collect().await.unwrap().to_bytes().as_ref())?;
    assert_eq!(outages.len(), 2);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn get() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(None, None, "get()", None, None).await?;
    pool.create_unresolved_site_outage(Some(1), None).await?;

    let response = client
      .oneshot(Request::builder().uri("/api/sites/outages/dd9a531a-1b0b-4a12-bc09-e5637f916261").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let outages: SiteOutage = serde_json::from_slice(response.into_body().collect().await.unwrap().to_bytes().as_ref())?;
    assert_eq!(&outages.uuid, "dd9a531a-1b0b-4a12-bc09-e5637f916261");

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn get_not_found() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(None, None, "get_not_found()", None, None).await?;
    pool.create_unresolved_site_outage(Some(1), None).await?;

    let response = client.oneshot(Request::builder().uri("/api/sites/outages/nonexistant").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    pool.cleanup().await;

    Ok(())
  }
}
