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
  api::{auth::Auth, error::Shortable, types::UserPatch, ApiResponse},
  ext::Run,
  model::User,
};

use super::error::{check_json, AppError};

pub async fn list(_: Auth, pool: State<Pool<MySql>>) -> ApiResponse<Json<Vec<User>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let users = User::list(&mut conn).await.context("could not retrieve users").short()?;

  Ok(Json(users))
}

pub async fn get(_: Auth, pool: State<Pool<MySql>>, Path(uuid): Path<String>) -> ApiResponse<Json<User>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let user = User::by_uuid(&mut conn, &uuid).await.context("could not retrieve user").short()?;

  Ok(Json(user))
}

pub async fn create(_: Auth, pool: State<Pool<MySql>>, payload: Result<Json<User>, JsonRejection>) -> ApiResponse<impl IntoResponse> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let mut payload = check_json(payload).short()?;
  payload.uuid = Uuid::new_v4().to_string();
  payload.insert(&mut conn).await.short()?;

  Ok((StatusCode::CREATED, [(header::LOCATION, format!("/api/users/{}", payload.uuid))]))
}

pub async fn update(_: Auth, pool: State<Pool<MySql>>, Path(uuid): Path<String>, payload: Result<Json<User>, JsonRejection>) -> ApiResponse<()> {
  let payload = check_json(payload).short()?;

  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let user = User::by_uuid(&mut conn, &uuid).await.context("could not retrieve user").short()?;

  let user = User {
    email: payload.email,
    name: payload.name,
    password: payload.password,
    ..user
  };

  user.update(&mut conn, true).await.context("could not update user").short()?;

  Ok(())
}

pub async fn patch(_: Auth, pool: State<Pool<MySql>>, Path(uuid): Path<String>, payload: Result<Json<UserPatch>, JsonRejection>) -> ApiResponse<()> {
  let payload = check_json(payload).short()?;

  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let mut user = User::by_uuid(&mut conn, &uuid).await.context("could not retrieve user").short()?;
  let mut update_password = false;

  payload.email.run(|value| user.email = value);
  payload.name.run(|value| user.name = value);
  payload.password.run(|value| {
    user.password = value;
    update_password = true;
  });

  user.update(&mut conn, update_password).await.context("could not update user").short()?;

  Ok(())
}

pub async fn delete(auth: Auth, pool: State<Pool<MySql>>, Path(uuid): Path<String>) -> ApiResponse<StatusCode> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  if uuid == auth.user.uuid {
    Err(AppError::BadRequest).context("cannot delete your own user").short()?;
  }

  User::delete(&mut conn, &uuid).await.context("could not delete user").short()?;

  Ok(StatusCode::NO_CONTENT)
}
