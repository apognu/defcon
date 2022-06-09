use anyhow::Context;
use rocket::{
  response::status::{Created, NoContent},
  serde::json::{Error as JsonError, Json},
  State,
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

#[get("/api/groups")]
pub async fn list(_auth: Auth, pool: &State<Pool<MySql>>) -> ApiResponse<Json<Vec<Group>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let groups = Group::all(&mut conn).await.short()?;

  Ok(Json(groups))
}

#[get("/api/groups/<uuid>")]
pub async fn get(_auth: Auth, pool: &State<Pool<MySql>>, uuid: String) -> ApiResponse<Json<Group>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let group = Group::by_uuid(&mut conn, &uuid).await.context("could not retrieve check").short()?;

  Ok(Json(group))
}

#[post("/api/groups", data = "<payload>")]
pub async fn create(_auth: Auth, pool: &State<Pool<MySql>>, payload: Result<Json<Group>, JsonError<'_>>) -> ApiResponse<Created<String>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let payload = check_json(payload).short()?.0;
  let uuid = Uuid::new_v4().to_string();

  let group = Group {
    uuid: uuid.clone(),
    name: payload.name,
    ..Default::default()
  };

  let group = group.insert(&mut *conn).await.context("could not create group").short()?;

  Ok(Created::new(uri!(get(uuid = group.uuid)).to_string()))
}

#[put("/api/groups/<uuid>", data = "<payload>")]
pub async fn update(_auth: Auth, pool: &State<Pool<MySql>>, uuid: String, payload: Result<Json<Group>, JsonError<'_>>) -> ApiResponse<()> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let payload = check_json(payload).short()?.0;

  let group = Group::by_uuid(&mut *conn, &uuid).await.context("could not retrieve group").short()?;
  let group = Group { name: payload.name, ..group };

  group.update(&mut *conn).await.context("could not update group").short()?;

  Ok(())
}

#[delete("/api/groups/<uuid>")]
pub async fn delete(_auth: Auth, pool: &State<Pool<MySql>>, uuid: String) -> ApiResponse<NoContent> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  Group::delete(&mut conn, &uuid).await.context("could not delete group").short()?;

  Ok(NoContent)
}
