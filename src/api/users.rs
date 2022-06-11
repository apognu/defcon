use anyhow::Context;
use rocket::{
  response::status::{Created, NoContent},
  serde::json::{Error as JsonError, Json},
  State,
};
use sqlx::{MySql, Pool};
use uuid::Uuid;

use crate::{
  api::{auth::Auth, error::Shortable, types::UserPatch, ApiResponse},
  ext::Run,
  model::User,
};

use super::error::{check_json, AppError};

#[get("/api/users")]
pub async fn list(_auth: Auth, pool: &State<Pool<MySql>>) -> ApiResponse<Json<Vec<User>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let users = User::list(&mut *conn).await.context("could not retrieve users").short()?;

  Ok(Json(users))
}

#[get("/api/users/<uuid>")]
pub async fn get(_auth: Auth, pool: &State<Pool<MySql>>, uuid: String) -> ApiResponse<Json<User>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let user = User::by_uuid(&mut *conn, &uuid).await.context("could not retrieve user").short()?;

  Ok(Json(user))
}

#[post("/api/users", data = "<payload>")]
pub async fn create(_auth: Auth, pool: &State<Pool<MySql>>, payload: Result<Json<User>, JsonError<'_>>) -> ApiResponse<Created<String>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let mut payload = check_json(payload).short()?.0;
  payload.uuid = Uuid::new_v4().to_string();
  payload.insert(&mut *conn).await.short()?;

  Ok(Created::new(uri!(get(uuid = payload.uuid)).to_string()))
}

#[put("/api/users/<uuid>", data = "<payload>")]
pub async fn update(_auth: Auth, pool: &State<Pool<MySql>>, uuid: String, payload: Result<Json<User>, JsonError<'_>>) -> ApiResponse<()> {
  let payload = check_json(payload).short()?.0;

  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let user = User::by_uuid(&mut *conn, &uuid).await.context("could not retrieve user").short()?;

  let user = User {
    email: payload.email,
    name: payload.name,
    password: payload.password,
    ..user
  };

  user.update(&mut *conn, true).await.context("could not update user").short()?;

  Ok(())
}

#[patch("/api/users/<uuid>", data = "<payload>")]
pub async fn patch(_auth: Auth, pool: &State<Pool<MySql>>, uuid: String, payload: Result<Json<UserPatch>, JsonError<'_>>) -> ApiResponse<()> {
  let payload = check_json(payload).short()?.0;

  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let mut user = User::by_uuid(&mut *conn, &uuid).await.context("could not retrieve user").short()?;
  let mut update_password = false;

  payload.email.run(|value| user.email = value);
  payload.name.run(|value| user.name = value);
  payload.password.run(|value| {
    user.password = value;
    update_password = true;
  });

  user.update(&mut *conn, update_password).await.context("could not update user").short()?;

  Ok(())
}

#[delete("/api/users/<uuid>")]
pub async fn delete(auth: Auth, pool: &State<Pool<MySql>>, uuid: String) -> ApiResponse<NoContent> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  if uuid == auth.user.uuid {
    Err(AppError::BadRequest).context("cannot delete your own user").short()?;
  }

  User::delete(&mut conn, &uuid).await.context("could not delete user").short()?;

  Ok(NoContent)
}
