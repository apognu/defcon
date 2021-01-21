use anyhow::Context;
use rocket::{response::status::Created, State};
use rocket_contrib::json::{Json, JsonError};
use sqlx::{MySql, Pool};
use uuid::Uuid;

use crate::{
  api::{
    error::{check_json, Errorable},
    ApiResponse,
  },
  model as db,
};

#[get("/api/alerters")]
pub async fn list(pool: State<'_, Pool<MySql>>) -> ApiResponse<Json<Vec<db::Alerter>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").apierr()?;
  let alerters = db::Alerter::all(&mut conn).await.apierr()?;

  Ok(Json(alerters))
}

#[get("/api/alerters/<uuid>")]
pub async fn get(pool: State<'_, Pool<MySql>>, uuid: String) -> ApiResponse<Json<db::Alerter>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").apierr()?;
  let alerter = db::Alerter::by_uuid(&mut conn, &uuid).await.apierr()?;

  Ok(Json(alerter))
}

#[post("/api/alerters", data = "<payload>")]
pub async fn add(pool: State<'_, Pool<MySql>>, payload: Result<Json<db::Alerter>, JsonError<'_>>) -> ApiResponse<Created<String>> {
  let payload = check_json(payload).apierr()?.0;
  let uuid = Uuid::new_v4().to_string();

  let alerter = db::Alerter {
    uuid: uuid.clone(),
    kind: payload.kind,
    webhook: payload.webhook,
    ..Default::default()
  };

  let mut conn = pool.acquire().await.context("could not retrieve database connection").apierr()?;
  let alerter = alerter.insert(&mut conn).await.apierr()?;

  Ok(Created::new(uri!(get: alerter.uuid).to_string()))
}

#[put("/api/alerters/<uuid>", data = "<payload>")]
pub async fn update(pool: State<'_, Pool<MySql>>, uuid: String, payload: Result<Json<db::Alerter>, JsonError<'_>>) -> ApiResponse<()> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").apierr()?;
  let alerter = db::Alerter::by_uuid(&mut conn, &uuid).await.apierr()?;
  let payload = check_json(payload).apierr()?.0;

  let alerter = db::Alerter {
    kind: payload.kind,
    webhook: payload.webhook,
    ..alerter
  };

  alerter.update(&mut conn).await.apierr()?;

  Ok(())
}
