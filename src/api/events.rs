use anyhow::Context;
use rocket::State;
use rocket_contrib::json::Json;
use sqlx::{MySql, Pool};

use crate::{
  api::{error::Shortable, ApiResponse},
  model as db,
};

#[get("/api/outages/<uuid>/events")]
pub async fn list(pool: State<'_, Pool<MySql>>, uuid: String) -> ApiResponse<Json<Vec<db::Event>>> {
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

    pool.create_check(None, None, "list()", None).await?;
    pool.create_unresolved_site_outage(None, None).await?;

    let response = client.get("/api/outages/dd9a531a-1b0b-4a12-bc09-e5637f916261/events").dispatch().await;
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

    let response = client.get("/api/outages/nonexistant/events").dispatch().await;
    assert_eq!(response.status(), Status::NotFound);

    pool.cleanup().await;

    Ok(())
  }
}
