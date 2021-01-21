use anyhow::Context;
use rocket::{
  response::status::{Created, NoContent},
  State,
};
use rocket_contrib::json::{Json, JsonError};
use sqlx::{MySql, Pool};
use uuid::Uuid;

use crate::{
  api::{
    error::{check_json, AppError, Errorable},
    types::{self as api, ApiMapper},
    ApiResponse,
  },
  ext::Run,
  model::{Alerter, Check},
};

#[get("/api/checks?<all>")]
pub async fn list(pool: State<'_, Pool<MySql>>, all: Option<bool>) -> ApiResponse<Json<Vec<api::Check>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").apierr()?;

  let checks = if all.is_some() {
    Check::all(&mut conn).await.apierr()?.map(&*pool).await.apierr()?
  } else {
    Check::enabled(&mut conn).await.apierr()?.map(&*pool).await.apierr()?
  };

  Ok(Json(checks))
}

#[get("/api/checks/<uuid>")]
pub async fn get(pool: State<'_, Pool<MySql>>, uuid: String) -> ApiResponse<Json<api::Check>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").apierr()?;
  let check = Check::by_uuid(&mut conn, &uuid).await.apierr()?.map(&*pool).await.apierr()?;

  Ok(Json(check))
}

#[post("/api/checks", data = "<payload>")]
pub async fn add(pool: State<'_, Pool<MySql>>, payload: Result<Json<api::Check>, JsonError<'_>>) -> ApiResponse<Created<String>> {
  let payload = check_json(payload).apierr()?.0;
  let uuid = Uuid::new_v4().to_string();

  let mut txn = pool.begin().await.context("could not start transaction").apierr()?;

  let alerter = match payload.alerter {
    Some(uuid) => Some(Alerter::by_uuid(&mut txn, &uuid).await.apierr()?),
    None => None,
  };

  let check = Check {
    uuid: uuid.clone(),
    alerter_id: alerter.map(|alerter| alerter.id),
    name: payload.check.name,
    enabled: payload.check.enabled,
    kind: payload.spec.kind(),
    interval: payload.check.interval,
    passing_threshold: payload.check.passing_threshold,
    failing_threshold: payload.check.failing_threshold,
    silent: payload.check.silent,
    ..Default::default()
  };

  let check = check.insert(&mut *txn).await.apierr()?;
  payload.spec.insert(&mut *txn, &check).await.apierr()?;

  txn.commit().await.context("could not commit transaction").apierr()?;

  Ok(Created::new(uri!(get: check.uuid).to_string()))
}

#[put("/api/checks/<uuid>", data = "<payload>")]
pub async fn update(pool: State<'_, Pool<MySql>>, uuid: String, payload: Result<Json<api::Check>, JsonError<'_>>) -> ApiResponse<()> {
  let payload = check_json(payload).apierr()?.0;

  let mut txn = pool.begin().await.context("could not start transaction").apierr()?;
  let check = Check::by_uuid(&mut txn, &uuid).await.apierr()?;

  if payload.spec.kind() != check.kind {
    Err(AppError::BadRequest(anyhow!("cannot change the resource `kind`"))).apierr()?;
  }

  let alerter = match payload.alerter {
    Some(uuid) => Some(Alerter::by_uuid(&mut txn, &uuid).await.apierr()?),
    None => None,
  };

  let check = Check {
    alerter_id: alerter.map(|alerter| alerter.id),
    enabled: payload.check.enabled,
    interval: payload.check.interval,
    passing_threshold: payload.check.passing_threshold,
    failing_threshold: payload.check.failing_threshold,
    silent: payload.check.silent,
    ..check
  };

  payload.spec.update(&mut *txn, &check).await.apierr()?;
  check.update(&mut *txn).await.apierr()?;

  txn.commit().await.context("could not commit transaction").apierr()?;

  Ok(())
}

#[patch("/api/checks/<uuid>", data = "<payload>")]
pub async fn patch(pool: State<'_, Pool<MySql>>, uuid: String, payload: Result<Json<api::CheckPatch>, JsonError<'_>>) -> ApiResponse<()> {
  let payload = check_json(payload).apierr()?.0;

  let mut txn = pool.begin().await.context("could not start transaction").apierr()?;
  let mut check = Check::by_uuid(&mut txn, &uuid).await.apierr()?;

  payload.name.run(|value| check.name = value);
  payload.enabled.run(|value| check.enabled = value);
  payload.interval.run(|value| check.interval = value);
  payload.passing_threshold.run(|value| check.passing_threshold = value);
  payload.failing_threshold.run(|value| check.failing_threshold = value);
  payload.silent.run(|value| check.silent = value);

  if let Some(value) = payload.alerter {
    let alerter = Alerter::by_uuid(&mut txn, &value).await.apierr()?;

    check.alerter_id = Some(alerter.id);
  }

  if let Some(value) = payload.spec {
    value.update(&mut *txn, &check).await.apierr()?;
  }

  check.update(&mut *txn).await.apierr()?;

  txn.commit().await.context("could not commit transaction").apierr()?;

  Ok(())
}

#[delete("/api/checks/<uuid>")]
pub async fn delete(pool: State<'_, Pool<MySql>>, uuid: String) -> ApiResponse<NoContent> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").apierr()?;
  Check::delete(&mut conn, &uuid).await.apierr()?;

  Ok(NoContent)
}
