use anyhow::Context;
use axum::{
  extract::{rejection::JsonRejection, Path, State},
  http::{header, StatusCode},
  response::IntoResponse,
  Json,
};
use sqlx::{MySql, Pool};
use uuid::Uuid;

use crate::{
  api::{
    auth::Auth,
    error::{check_json, Shortable},
    ApiResponse,
  },
  model::Group,
};

pub async fn list(_: Auth, pool: State<Pool<MySql>>) -> ApiResponse<Json<Vec<Group>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let groups = Group::all(&mut conn).await.short()?;

  Ok(Json(groups))
}

pub async fn get(_: Auth, pool: State<Pool<MySql>>, Path(uuid): Path<String>) -> ApiResponse<Json<Group>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let group = Group::by_uuid(&mut conn, &uuid).await.context("could not retrieve check").short()?;

  Ok(Json(group))
}

pub async fn create(_: Auth, pool: State<Pool<MySql>>, payload: Result<Json<Group>, JsonRejection>) -> ApiResponse<impl IntoResponse> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let payload = check_json(payload).short()?;
  let uuid = Uuid::new_v4().to_string();

  let group = Group {
    uuid: uuid.clone(),
    name: payload.name,
    ..Default::default()
  };

  let group = group.insert(&mut conn).await.context("could not create group").short()?;

  Ok((StatusCode::CREATED, [(header::LOCATION, format!("/api/groups/{}", group.uuid))]))
}

pub async fn update(_: Auth, pool: State<Pool<MySql>>, Path(uuid): Path<String>, payload: Result<Json<Group>, JsonRejection>) -> ApiResponse<()> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let payload = check_json(payload).short()?;

  let group = Group::by_uuid(&mut conn, &uuid).await.context("could not retrieve group").short()?;
  let group = Group { name: payload.name, ..group };

  group.update(&mut conn).await.context("could not update group").short()?;

  Ok(())
}

pub async fn delete(_: Auth, pool: State<Pool<MySql>>, Path(uuid): Path<String>) -> ApiResponse<impl IntoResponse> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  Group::delete(&mut conn, &uuid).await.context("could not delete group").short()?;

  Ok(StatusCode::NO_CONTENT)
}
