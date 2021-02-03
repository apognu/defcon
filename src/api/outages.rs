use anyhow::Context;
use rocket::State;
use rocket_contrib::json::{Json, JsonError};
use sqlx::{MySql, Pool};

use crate::{
  api::{
    error::{check_json, Errorable},
    types as api, ApiResponse,
  },
  model::Outage,
};

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

  use crate::tests;

  #[tokio::test]
  async fn comment() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(None, None, "comment()", None).await?;
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
