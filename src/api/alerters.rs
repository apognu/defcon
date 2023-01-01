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
  model as db,
};

pub async fn list(_: Auth, pool: State<Pool<MySql>>) -> ApiResponse<Json<Vec<db::Alerter>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let alerters = db::Alerter::all(&mut conn).await.context("could not retrieve alerters").short()?;

  Ok(Json(alerters))
}

pub async fn get(_: Auth, pool: State<Pool<MySql>>, Path(uuid): Path<String>) -> ApiResponse<Json<db::Alerter>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let alerter = db::Alerter::by_uuid(&mut conn, &uuid).await.context("could not find alerter").short()?;

  Ok(Json(alerter))
}

pub async fn add(_: Auth, pool: State<Pool<MySql>>, payload: Result<Json<db::Alerter>, JsonRejection>) -> ApiResponse<impl IntoResponse> {
  let payload = check_json(payload).short()?;
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

  Ok((StatusCode::CREATED, [(header::LOCATION, format!("/api/alerters/{}", alerter.uuid))]))
}

pub async fn update(_: Auth, pool: State<Pool<MySql>>, Path(uuid): Path<String>, payload: Result<Json<db::Alerter>, JsonRejection>) -> ApiResponse<()> {
  let payload = check_json(payload).short()?;
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

pub async fn delete(_: Auth, pool: State<Pool<MySql>>, Path(uuid): Path<String>) -> ApiResponse<StatusCode> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  db::Alerter::delete(&mut conn, &uuid).await.context("could not delete alerter").short()?;

  Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use axum::{
    body::Body,
    http::{Request, StatusCode},
  };
  use hyper::{body, Method};
  use serde_json::json;
  use tower::ServiceExt;

  use crate::{
    model::{Alerter, AlerterKind},
    tests,
  };

  #[tokio::test]
  async fn list() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_alerter().await?;

    let response = client.oneshot(Request::builder().uri("/api/alerters").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let checks: Vec<Alerter> = serde_json::from_slice(body::to_bytes(response.into_body()).await.unwrap().as_ref())?;
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

    let response = client
      .oneshot(Request::builder().uri("/api/alerters/dd9a531a-1b0b-4a12-bc09-e5637f916261").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let alerter: Alerter = serde_json::from_slice(body::to_bytes(response.into_body()).await.unwrap().as_ref())?;
    assert_eq!(alerter.kind, AlerterKind::Webhook);
    assert!(matches!(alerter.url.as_deref(), Some("https://webhooks.example.com/1")));

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn get_not_found() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_alerter().await?;

    let response = client.oneshot(Request::builder().uri("/api/alerters/nonexistant").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

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

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/alerters")
          .header("content-type", "application/json")
          .body(Body::from(payload.to_string()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

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

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/alerters")
          .header("content-type", "application/json")
          .body(Body::from(payload.to_string()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

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

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::PUT)
          .uri("/api/alerters/dd9a531a-1b0b-4a12-bc09-e5637f916261")
          .header("content-type", "application/json")
          .body(Body::from(payload.to_string()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

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

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::PUT)
          .uri("/api/alerters/dd9a531a-1b0b-4a12-bc09-e5637f916261")
          .header("content-type", "application/json")
          .body(Body::from(payload.to_string()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    pool.cleanup().await;

    Ok(())
  }
}
