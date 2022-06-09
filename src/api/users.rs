use anyhow::Context;
use rocket::{
  response::status::Created,
  serde::json::{Error as JsonError, Json},
  State,
};
use sqlx::{MySql, Pool};
use uuid::Uuid;

use crate::{
  api::{auth::Auth, error::Shortable, ApiResponse},
  model::User,
};

use super::error::check_json;

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
