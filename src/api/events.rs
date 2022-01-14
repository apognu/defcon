use anyhow::Context;
use rocket::{serde::json::Json, State};
use sqlx::{MySql, Pool};

use crate::{
  api::{error::Shortable, types as api, ApiResponse},
  model as db,
};

#[get("/api/checks/<uuid>/events", rank = 10)]
pub async fn list_for_check(pool: &State<Pool<MySql>>, uuid: String) -> ApiResponse<Json<Vec<db::Event>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let check = db::Check::by_uuid(&mut conn, &uuid).await.context("could not retrieve check").short()?;

  let events = db::Event::for_check(&mut conn, &check).await.context("could not retrieve events").short()?;

  Ok(Json(events))
}

#[get("/api/checks/<uuid>/events?<from>&<to>", rank = 5)]
pub async fn list_for_check_between(pool: &State<Pool<MySql>>, uuid: String, from: api::DateTime, to: api::DateTime) -> ApiResponse<Json<Vec<db::Event>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let check = db::Check::by_uuid(&mut conn, &uuid).await.context("could not retrieve check").short()?;

  let events = db::Event::for_check_between(&mut conn, &check, *from, *to).await.context("could not retrieve events").short()?;

  Ok(Json(events))
}

#[get("/api/sites/outages/<uuid>/events")]
pub async fn list_for_outage(pool: &State<Pool<MySql>>, uuid: String) -> ApiResponse<Json<Vec<db::Event>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let outage = db::SiteOutage::by_uuid(&mut conn, &uuid).await.context("could not retrieve outage").short()?;

  let events = db::Event::for_outage(&mut conn, &outage).await.context("could not retrieve events").short()?;

  Ok(Json(events))
}

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use rocket::http::Status;

  use crate::{model::Event, tests};

  #[tokio::test]
  async fn list() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(None, None, "list()", None, None).await?;
    pool.create_unresolved_site_outage(None, None).await?;

    let response = client.get("/api/sites/outages/dd9a531a-1b0b-4a12-bc09-e5637f916261/events").dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    let events: Vec<Event> = serde_json::from_str(&response.into_string().await.unwrap())?;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].status, 1);
    assert_eq!(&events[0].message, "failure");

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn list_not_found() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    let response = client.get("/api/sites/outages/nonexistant/events").dispatch().await;
    assert_eq!(response.status(), Status::NotFound);

    pool.cleanup().await;

    Ok(())
  }
}
