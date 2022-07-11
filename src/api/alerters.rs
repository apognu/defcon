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
  model as db,
};

#[get("/api/alerters")]
pub async fn list(_auth: Auth, pool: &State<Pool<MySql>>) -> ApiResponse<Json<Vec<db::Alerter>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let alerters = db::Alerter::all(&mut conn).await.context("could not retrieve alerters").short()?;

  Ok(Json(alerters))
}

#[get("/api/alerters/<uuid>")]
pub async fn get(_auth: Auth, pool: &State<Pool<MySql>>, uuid: String) -> ApiResponse<Json<db::Alerter>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let alerter = db::Alerter::by_uuid(&mut conn, &uuid).await.context("could not find alerter").short()?;

  Ok(Json(alerter))
}

#[post("/api/alerters", data = "<payload>")]
pub async fn add(_auth: Auth, pool: &State<Pool<MySql>>, payload: Result<Json<db::Alerter>, JsonError<'_>>) -> ApiResponse<Created<String>> {
  let payload = check_json(payload).short()?.0;
  let uuid = Uuid::new_v4().to_string();

  let alerter = db::Alerter {
    uuid: uuid.clone(),
    name: payload.name,
    kind: payload.kind,
    url: payload.url,
    username: payload.username,
    password: payload.password,
    ..Default::default()
  };

  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let alerter = alerter.insert(&mut conn).await.context("could not create alerter").short()?;

  Ok(Created::new(uri!(get(uuid = alerter.uuid)).to_string()))
}

#[put("/api/alerters/<uuid>", data = "<payload>")]
pub async fn update(_auth: Auth, pool: &State<Pool<MySql>>, uuid: String, payload: Result<Json<db::Alerter>, JsonError<'_>>) -> ApiResponse<()> {
  let payload = check_json(payload).short()?.0;
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let alerter = db::Alerter::by_uuid(&mut conn, &uuid).await.context("could not find alerter").short()?;

  let alerter = db::Alerter {
    name: payload.name,
    kind: payload.kind,
    url: payload.url,
    username: payload.username,
    password: payload.password,
    ..alerter
  };

  alerter.update(&mut conn).await.context("could not update alerter").short()?;

  Ok(())
}

#[delete("/api/alerters/<uuid>")]
pub async fn delete(_auth: Auth, pool: &State<Pool<MySql>>, uuid: String) -> ApiResponse<NoContent> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  db::Alerter::delete(&mut conn, &uuid).await.context("could not delete alerter").short()?;

  Ok(NoContent)
}

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use rocket::{http::Status, serde::json::json};

  use crate::{
    model::{Alerter, AlerterKind},
    tests,
  };

  #[tokio::test]
  async fn list() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_alerter().await?;

    let response = client.get("/api/alerters").dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    let checks: Vec<Alerter> = serde_json::from_str(&response.into_string().await.unwrap())?;
    assert_eq!(checks.len(), 1);
    assert_eq!(checks[0].kind, AlerterKind::Webhook);
    assert!(matches!(checks[0].url.as_deref(), Some("https://webhooks.example.com/1")));

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn get() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_alerter().await?;

    let response = client.get("/api/alerters/dd9a531a-1b0b-4a12-bc09-e5637f916261").dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    let alerter: Alerter = serde_json::from_str(&response.into_string().await.unwrap())?;
    assert_eq!(alerter.kind, AlerterKind::Webhook);
    assert!(matches!(alerter.url.as_deref(), Some("https://webhooks.example.com/1")));

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn get_not_found() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_alerter().await?;

    let response = client.get("/api/alerters/nonexistant").dispatch().await;
    assert_eq!(response.status(), Status::NotFound);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn create() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    let payload = json!({
      "name": "My Alerter",
      "kind": "webhook",
      "url": "https://hooks.example.com/1"
    });

    let response = client.post("/api/alerters").body(payload.to_string().as_bytes()).dispatch().await;
    assert_eq!(response.status(), Status::Created);

    let check = sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>)>("SELECT name, kind, url, username, password FROM alerters")
      .fetch_one(&*pool)
      .await?;

    assert_eq!(&check.0, "My Alerter");
    assert_eq!(&check.1, "webhook");
    assert_eq!(&check.2, "https://hooks.example.com/1");
    assert!(matches!(check.3, None));
    assert!(matches!(check.4, None));

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn create_with_credentials() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    let payload = json!({
      "name": "My Alerter",
      "kind": "webhook",
      "url": "https://hooks.example.com/1",
      "username": "bob",
      "password": "password"
    });

    let response = client.post("/api/alerters").body(payload.to_string().as_bytes()).dispatch().await;
    assert_eq!(response.status(), Status::Created);

    let check = sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>)>("SELECT name, kind, url, username, password FROM alerters")
      .fetch_one(&*pool)
      .await?;

    assert_eq!(&check.0, "My Alerter");
    assert_eq!(&check.1, "webhook");
    assert_eq!(&check.2, "https://hooks.example.com/1");
    assert!(matches!(check.3.as_deref(), Some("bob")));
    assert!(matches!(check.4.as_deref(), Some("password")));

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn update() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_alerter().await?;

    let payload = json!({
      "name": "My Alerter",
      "kind": "webhook",
      "url": "https://hooks.example.com/2",
      "username": "bob",
    });

    let response = client.put("/api/alerters/dd9a531a-1b0b-4a12-bc09-e5637f916261").body(payload.to_string().as_bytes()).dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    let check = sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>)>("SELECT name, kind, url, username, password FROM alerters")
      .fetch_one(&*pool)
      .await?;

    assert_eq!(&check.0, "My Alerter");
    assert_eq!(&check.1, "webhook");
    assert_eq!(&check.2, "https://hooks.example.com/2");
    assert!(matches!(check.3.as_deref(), Some("bob")));
    assert!(matches!(check.4.as_deref(), None));

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn update_bad_request() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_alerter().await?;

    let payload = json!({
      "kind": "hello",
      "url": "https://hooks.example.com/2"
    });

    let response = client.put("/api/alerters/dd9a531a-1b0b-4a12-bc09-e5637f916261").body(payload.to_string().as_bytes()).dispatch().await;
    assert_eq!(response.status(), Status::BadRequest);

    pool.cleanup().await;

    Ok(())
  }
}
