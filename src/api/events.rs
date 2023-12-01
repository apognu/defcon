use anyhow::Context;
use axum::{
  extract::{Path, Query, State},
  Json,
};
use sqlx::{MySql, Pool};

use crate::{
  api::{auth::Auth, error::Shortable, types as api, ApiResponse},
  model as db,
};

#[derive(Deserialize)]
pub struct ListForCheckQuery {
  from: Option<api::DateTime>,
  to: Option<api::DateTime>,
  limit: Option<u8>,
  page: Option<u8>,
}

pub async fn list_for_check(
  _: Auth,
  pool: State<Pool<MySql>>,
  Path(uuid): Path<String>,
  Query(ListForCheckQuery { from, to, limit, page }): Query<ListForCheckQuery>,
) -> ApiResponse<Json<Vec<db::Event>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let check = db::Check::by_uuid(&mut conn, &uuid).await.context("could not retrieve check").short()?;

  let events = if let Some((from, to)) = from.zip(to) {
    db::Event::for_check_between(&mut conn, &check, *from, *to, limit, page)
      .await
      .context("could not retrieve events")
      .short()?
  } else {
    db::Event::for_check(&mut conn, &check, limit, page).await.context("could not retrieve events").short()?
  };

  Ok(Json(events))
}

#[derive(Deserialize)]
pub struct ListForOutageQuery {
  limit: Option<u8>,
  page: Option<u8>,
}

pub async fn list_for_outage(_: Auth, pool: State<Pool<MySql>>, Path(uuid): Path<String>, Query(ListForOutageQuery { limit, page }): Query<ListForOutageQuery>) -> ApiResponse<Json<Vec<db::Event>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let outage = db::Outage::by_uuid(&mut conn, &uuid).await.context("could not retrieve outage").short()?;

  let events = db::Event::for_outage(&mut conn, &outage, limit, page).await.context("could not retrieve events").short()?;

  Ok(Json(events))
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

  use crate::{model::Event, tests};

  #[tokio::test]
  async fn list() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(None, None, "list()", None, None).await?;
    pool.create_unresolved_site_outage(None, None).await?;
    pool.create_resolved_outage(None, None).await?;

    let response = client
      .oneshot(Request::builder().uri("/api/outages/dd9a531a-1b0b-4a12-bc09-e5637f916261/events").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let events: Vec<Event> = serde_json::from_slice(response.into_body().collect().await.unwrap().to_bytes().as_ref())?;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].status, 1);
    assert_eq!(&events[0].message, "failure");

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn list_not_found() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    let response = client.oneshot(Request::builder().uri("/api/outages/nonexistant/events").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    pool.cleanup().await;

    Ok(())
  }
}
