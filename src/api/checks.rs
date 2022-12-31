use std::sync::Arc;

use anyhow::Context;
use axum::{
  extract::{rejection::JsonRejection, Path, Query, State},
  http::{header, StatusCode},
  response::IntoResponse,
  Json,
};
use sqlx::{MySql, Pool};
use uuid::Uuid;

use crate::{
  api::{
    error::{check_json, AppError, Shortable},
    types::{self as api, ApiMapper, Sites},
    ApiResponse,
  },
  config::{Config, CONTROLLER_ID},
  ext::Run,
  model::{Alerter, Check, CheckKind, Group},
};

use super::auth::Auth;

#[derive(Deserialize)]
pub struct ListQuery {
  all: Option<bool>,
  group: Option<String>,
  kind: Option<String>,
  site: Option<String>,
}

pub async fn list(_: Auth, ref pool: State<Pool<MySql>>, Query(ListQuery { all, group, kind, site }): Query<ListQuery>) -> ApiResponse<Json<Vec<api::Check>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let group = match group {
    None => None,
    Some(uuid) => Some(Group::by_uuid(&mut conn, &uuid).await.context("could not retrieve group").short()?),
  };

  let kind = match kind {
    None => None,
    Some(kind) => Some(CheckKind::try_from(kind).context("cannot filter by this handler type").short()?),
  };

  let checks = Check::list(&mut conn, all.unwrap_or(false), false, group, kind, site)
    .await
    .short()?
    .map(pool)
    .await
    .context("could not retrieve checks")
    .short()?;

  Ok(Json(checks))
}

pub async fn get(_: Auth, ref pool: State<Pool<MySql>>, Path(uuid): Path<String>) -> ApiResponse<Json<api::Check>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let check = Check::by_uuid(&mut conn, &uuid).await.context("could not retrieve check").short()?.map(pool).await.short()?;

  Ok(Json(check))
}

pub async fn create(_: Auth, config: State<Arc<Config>>, pool: State<Pool<MySql>>, payload: Result<Json<api::Check>, JsonRejection>) -> ApiResponse<impl IntoResponse> {
  let payload = check_json(payload).short()?;
  let uuid = Uuid::new_v4().to_string();

  let sites = match payload.sites {
    Some(sites) => sites,
    None => Sites(vec![CONTROLLER_ID.to_string()]),
  };

  if payload.check.site_threshold as usize > sites.len() {
    Err(anyhow!("`site_threshold` cannot exceed the number of `sites`")).context(AppError::BadRequest).short()?;
  }

  let mut txn = pool.begin().await.context("could not start transaction").short()?;

  let group = match payload.group_in {
    Some(ref group) => Some(Group::by_uuid(&mut txn, group).await.context("could not retrieve group").short()?),
    None => None,
  };

  let alerter = match payload.alerter_in {
    Some(uuid) => Some(Alerter::by_uuid(&mut txn, &uuid).await.context("could not retrieve alerter").short()?),
    None => match config.alerters.default.as_ref() {
      Some(uuid) => Some(Alerter::by_uuid(&mut txn, uuid).await.context("could not retrieve alerter").short()?),
      None => None,
    },
  };

  let check = Check {
    uuid: uuid.clone(),
    group_id: group.map(|group| group.id),
    alerter_id: alerter.map(|alerter| alerter.id),
    name: payload.check.name,
    enabled: payload.check.enabled,
    on_status_page: payload.check.on_status_page,
    kind: payload.spec.kind(),
    interval: payload.check.interval,
    site_threshold: payload.check.site_threshold,
    passing_threshold: payload.check.passing_threshold,
    failing_threshold: payload.check.failing_threshold,
    silent: payload.check.silent,
    ..Default::default()
  };

  let check = check.insert(&mut txn).await.context("could not create check").short()?;
  payload.spec.insert(&mut txn, &check).await.context("could not create spec").short()?;
  check.update_sites(&mut txn, &sites).await.context("could not update check sites").short()?;

  txn.commit().await.context("could not commit transaction").short()?;

  Ok((StatusCode::CREATED, [(header::LOCATION, format!("/api/checks/{}", check.uuid))]))
}

pub async fn update(_: Auth, pool: State<Pool<MySql>>, Path(uuid): Path<String>, payload: Result<Json<api::Check>, JsonRejection>) -> ApiResponse<()> {
  let payload = check_json(payload).short()?;

  let sites = match payload.sites {
    Some(sites) => sites,
    None => Sites(vec![CONTROLLER_ID.to_string()]),
  };

  if payload.check.site_threshold as usize > sites.len() {
    Err(anyhow!("`site_threshold` cannot exceed the number of `sites`").context(AppError::BadRequest)).short()?;
  }

  let mut txn = pool.begin().await.context("could not start transaction").short()?;
  let check = Check::by_uuid(&mut txn, &uuid).await.context("could not retrieve check").short()?;

  if payload.spec.kind() != check.kind {
    Err(anyhow!("cannot change the resource `kind`").context(AppError::BadRequest)).short()?;
  }

  let group = match payload.group_in {
    Some(group) => Some(Group::by_uuid(&mut txn, &group).await.context("could not retrieve group").short()?),
    None => None,
  };

  let alerter = match payload.alerter_in {
    Some(uuid) => Some(Alerter::by_uuid(&mut txn, &uuid).await.context("could not retrieve alerter").short()?),
    None => None,
  };

  let check = Check {
    name: payload.check.name,
    group_id: group.map(|group| group.id),
    alerter_id: alerter.map(|alerter| alerter.id),
    enabled: payload.check.enabled,
    on_status_page: payload.check.on_status_page,
    interval: payload.check.interval,
    site_threshold: payload.check.site_threshold,
    passing_threshold: payload.check.passing_threshold,
    failing_threshold: payload.check.failing_threshold,
    silent: payload.check.silent,
    ..check
  };

  payload.spec.update(&mut txn, &check).await.context("could not update spec").short()?;
  check.update_sites(&mut txn, &sites).await.context("could not update check sites").short()?;
  check.update(&mut txn).await.context("could not update check").short()?;

  txn.commit().await.context("could not commit transaction").short()?;

  Ok(())
}

pub async fn patch(_: Auth, pool: State<Pool<MySql>>, Path(uuid): Path<String>, payload: Result<Json<api::CheckPatch>, JsonRejection>) -> ApiResponse<()> {
  let payload = check_json(payload).short()?;

  let mut txn = pool.begin().await.context("could not start transaction").short()?;
  let mut check = Check::by_uuid(&mut txn, &uuid).await.context("could not retrieve check").short()?;

  payload.name.run(|value| check.name = value);
  payload.enabled.run(|value| check.enabled = value);
  payload.on_status_page.run(|value| check.on_status_page = value);
  payload.interval.run(|value| check.interval = value);
  payload.site_threshold.run(|value| check.site_threshold = value);
  payload.passing_threshold.run(|value| check.passing_threshold = value);
  payload.failing_threshold.run(|value| check.failing_threshold = value);
  payload.silent.run(|value| check.silent = value);

  if let Some(value) = payload.group {
    let group = Group::by_uuid(&mut txn, &value).await.context("could not retrieve group").short()?;

    check.group_id = Some(group.id);
  } else {
    check.group_id = None;
  }

  if let Some(value) = payload.alerter {
    if value.is_empty() {
      check.alerter_id = None;
    } else {
      let alerter = Alerter::by_uuid(&mut txn, &value).await.context("could not retrieve alerter").short()?;

      check.alerter_id = Some(alerter.id);
    }
  }

  if let Some(value) = payload.sites {
    check.update_sites(&mut txn, &value).await.short()?;
  }

  if let Some(value) = payload.spec {
    value.update(&mut txn, &check).await.short()?;
  }

  check.update(&mut txn).await.short()?;

  let check = Check::by_uuid(&mut txn, &uuid).await.short()?;
  let sites = check.sites(&mut txn).await.short()?;

  if check.site_threshold as usize > sites.len() {
    Err(anyhow!("`site_threshold` cannot exceed the number of `sites`").context(AppError::BadRequest)).short()?;
  }

  txn.commit().await.context("could not commit transaction").short()?;

  Ok(())
}

#[derive(Deserialize)]
pub struct DeleteQuery {
  delete: Option<bool>,
}

pub async fn delete(_: Auth, pool: State<Pool<MySql>>, Path(uuid): Path<String>, Query(DeleteQuery { delete }): Query<DeleteQuery>) -> ApiResponse<impl IntoResponse> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  match delete.unwrap_or(false) {
    false => Check::disable(&mut conn, &uuid).await.context("could not disable check").short()?,
    true => Check::delete(&mut conn, &uuid).await.context("could not delete check").short()?,
  }

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
  use tower::{Service, ServiceExt};
  use uuid::Uuid;

  use crate::{api::types as api, config::CONTROLLER_ID, tests};

  #[tokio::test]
  async fn list() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(Some(1), Some(Uuid::new_v4().to_string()), "list_checks_1()", Some(true), None).await?;
    pool.create_check(Some(2), Some(Uuid::new_v4().to_string()), "list_checks_2()", Some(false), None).await?;

    let response = client.oneshot(Request::builder().uri("/api/checks").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let checks: Vec<api::Check> = serde_json::from_slice(body::to_bytes(response.into_body()).await.unwrap().as_ref())?;
    assert_eq!(checks.len(), 1);
    assert_eq!(&checks[0].check.name, "list_checks_1()");

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn list_all() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(Some(1), Some(Uuid::new_v4().to_string()), "list_checks_1()", Some(true), None).await?;
    pool.create_check(Some(2), Some(Uuid::new_v4().to_string()), "list_checks_2()", Some(false), None).await?;

    let response = client.oneshot(Request::builder().uri("/api/checks?all=true").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let checks: Vec<api::Check> = serde_json::from_slice(body::to_bytes(response.into_body()).await.unwrap().as_ref())?;
    assert_eq!(checks.len(), 2);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn list_by_kind() -> Result<()> {
    let (pool, mut client) = tests::api_client().await?;

    pool.create_check(Some(1), Some(Uuid::new_v4().to_string()), "list_checks_1()", Some(true), None).await?;

    let response = client
      .ready()
      .await
      .unwrap()
      .call(Request::builder().uri("/api/checks?kind=tcp").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let checks: Vec<api::Check> = serde_json::from_slice(body::to_bytes(response.into_body()).await.unwrap().as_ref())?;
    assert_eq!(checks.len(), 1);

    let response = client
      .ready()
      .await
      .unwrap()
      .call(Request::builder().uri("/api/checks?kind=http").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let checks: Vec<api::Check> = serde_json::from_slice(body::to_bytes(response.into_body()).await.unwrap().as_ref())?;
    assert_eq!(checks.len(), 0);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn list_by_site() -> Result<()> {
    let (pool, mut client) = tests::api_client().await?;

    pool.create_check(Some(1), Some(Uuid::new_v4().to_string()), "list_checks_1()", Some(true), Some(&["eu-1"])).await?;

    let response = client
      .ready()
      .await
      .unwrap()
      .call(Request::builder().uri("/api/checks?site=eu-1").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let checks: Vec<api::Check> = serde_json::from_slice(body::to_bytes(response.into_body()).await.unwrap().as_ref())?;
    assert_eq!(checks.len(), 1);

    let response = client
      .ready()
      .await
      .unwrap()
      .call(Request::builder().uri("/api/checks?site=nosite").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let checks: Vec<api::Check> = serde_json::from_slice(body::to_bytes(response.into_body()).await.unwrap().as_ref())?;
    assert_eq!(checks.len(), 0);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn get() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(None, None, "get_check()", None, None).await?;

    let response = client
      .oneshot(Request::builder().uri("/api/checks/dd9a531a-1b0b-4a12-bc09-e5637f916261").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let check: api::Check = serde_json::from_slice(body::to_bytes(response.into_body()).await.unwrap().as_ref())?;
    assert_eq!(&check.check.name, "get_check()");

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn get_not_found() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    let response = client.oneshot(Request::builder().uri("/api/checks/nonexistant").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn create() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    let check = json!({
      "name": "create()",
      "enabled": false,
      "interval": "10s",
      "sites": [CONTROLLER_ID],
      "site_threshold": 1,
      "passing_threshold": 1,
      "failing_threshold": 1,
      "spec": {
        "kind": "app_store",
        "bundle_id": "helloworld"
      }
    });

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/checks")
          .header("content-type", "application/json")
          .body(Body::from(check.to_string()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let checks = sqlx::query_as::<_, (String,)>("SELECT name FROM checks").fetch_all(&*pool).await?;

    assert_eq!(checks.len(), 1);
    assert_eq!(&checks[0].0, "create()");

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn create_invalid_kind() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    let check = json!({
      "name": "create_invalid_kind()",
      "enabled": false,
      "interval": "10s",
      "sites": [CONTROLLER_ID],
      "site_threshold": 1,
      "passing_threshold": 1,
      "failing_threshold": 1,
      "spec": {
        "kind": "nonexistant",
        "bundle_id": "helloworld"
      }
    });

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/checks")
          .header("content-type", "application/json")
          .body(Body::from(check.to_string()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn create_invalid_spec() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    let check = json!({
      "name": "create_invalid_spec()",
      "enabled": false,
      "interval": "10s",
      "sites": [CONTROLLER_ID],
      "site_threshold": 1,
      "passing_threshold": 1,
      "failing_threshold": 1,
      "spec": {
        "kind": "http",
        "bundle_id": "helloworld"
      }
    });

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/checks")
          .header("content-type", "application/json")
          .body(Body::from(check.to_string()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn create_not_enough_sites() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    let check = json!({
      "name": "create_not_enough_sites()",
      "enabled": false,
      "interval": "10s",
      "sites": [CONTROLLER_ID],
      "site_threshold": 2,
      "passing_threshold": 1,
      "failing_threshold": 1,
      "spec": {
        "kind": "app_store",
        "bundle_id": "helloworld"
      }
    });

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/checks")
          .header("content-type", "application/json")
          .body(Body::from(check.to_string()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn create_bad_request() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    let check = json!({
      "name": "create_bad_request()",
      "enabled": false,
      "interval": "10s",
      "passing_threshold": 1,
      "failing_threshold": 1
    });

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/checks")
          .header("content-type", "application/json")
          .body(Body::from(check.to_string()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn update() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(None, None, "update()", None, None).await?;

    let check = json!({
      "name": "new_update()",
      "enabled": false,
      "interval": "15s",
      "sites": [CONTROLLER_ID],
      "site_threshold": 1,
      "passing_threshold": 1,
      "failing_threshold": 1,
      "spec": {
        "kind": "tcp",
        "host": "1.2.3.4",
        "port": 81,
        "timeout": "1h"
      }
    });

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::PUT)
          .uri("/api/checks/dd9a531a-1b0b-4a12-bc09-e5637f916261")
          .header("content-type", "application/json")
          .body(Body::from(check.to_string()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let check = sqlx::query_as::<_, (String, bool, u64)>(r#"SELECT name, enabled, `interval` FROM checks WHERE uuid = "dd9a531a-1b0b-4a12-bc09-e5637f916261""#)
      .fetch_one(&*pool)
      .await?;

    let spec = sqlx::query_as::<_, (String, u16, u64)>(r#"SELECT host, port, timeout FROM tcp_specs WHERE check_id = 1"#)
      .fetch_one(&*pool)
      .await?;

    assert_eq!(&check.0, "new_update()");
    assert_eq!(check.1, false);
    assert_eq!(check.2, 15);
    assert_eq!(spec.0, "1.2.3.4");
    assert_eq!(spec.1, 81);
    assert_eq!(spec.2, 3600);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn update_different_kind() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(None, None, "update_different_kind()", None, None).await?;

    let check = json!({
      "name": "update_different_kind()",
      "enabled": false,
      "interval": "15s",
      "passing_threshold": 1,
      "failing_threshold": 1,
      "spec": {
        "kind": "http",
        "url": "https://example.com"
      }
    });

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::PUT)
          .uri("/api/checks/dd9a531a-1b0b-4a12-bc09-e5637f916261")
          .header("content-type", "application/json")
          .body(Body::from(check.to_string()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn update_not_enough_sites() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    let check = json!({
      "name": "update_not_enough_sites()",
      "enabled": false,
      "interval": "10s",
      "sites": [CONTROLLER_ID],
      "site_threshold": 2,
      "passing_threshold": 1,
      "failing_threshold": 1,
      "spec": {
        "kind": "app_store",
        "bundle_id": "helloworld"
      }
    });

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::PUT)
          .uri("/api/checks/dd9a531a-1b0b-4a12-bc09-e5637f916261")
          .header("content-type", "application/json")
          .body(Body::from(check.to_string()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn patch() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(None, None, "patch()", None, None).await?;

    let check = json!({
      "name": "new_patch()",
      "interval": "10m",
      "enabled": false
    });

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::PATCH)
          .uri("/api/checks/dd9a531a-1b0b-4a12-bc09-e5637f916261")
          .header("content-type", "application/json")
          .body(Body::from(check.to_string()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let check = sqlx::query_as::<_, (String, bool, u64)>(r#"SELECT name, enabled, `interval` FROM checks WHERE uuid = "dd9a531a-1b0b-4a12-bc09-e5637f916261""#)
      .fetch_one(&*pool)
      .await?;

    let spec = sqlx::query_as::<_, (String, u16, u64)>(r#"SELECT host, port, timeout FROM tcp_specs WHERE check_id = 1"#)
      .fetch_one(&*pool)
      .await?;

    assert_eq!(&check.0, "new_patch()");
    assert_eq!(check.1, false);
    assert_eq!(check.2, 600);
    assert_eq!(spec.0, "0.0.0.0");
    assert_eq!(spec.1, 80);
    assert_eq!(spec.2, 10);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn patch_not_enough_sites() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(None, None, "patch_not_enough_sites()", None, None).await?;

    let check = json!({
      "sites": [CONTROLLER_ID],
      "site_threshold": 2
    });

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::PATCH)
          .uri("/api/checks/dd9a531a-1b0b-4a12-bc09-e5637f916261")
          .header("content-type", "application/json")
          .body(Body::from(check.to_string()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn disable() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(None, None, "disable()", None, None).await?;

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::DELETE)
          .uri("/api/checks/dd9a531a-1b0b-4a12-bc09-e5637f916261")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let check = sqlx::query_as::<_, (bool,)>(r#"SELECT enabled FROM checks WHERE uuid = "dd9a531a-1b0b-4a12-bc09-e5637f916261""#)
      .fetch_one(&*pool)
      .await?;

    assert_eq!(check.0, false);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn delete() -> Result<()> {
    let (pool, client) = tests::api_client().await?;

    pool.create_check(None, None, "delete()", None, None).await?;

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::DELETE)
          .uri("/api/checks/dd9a531a-1b0b-4a12-bc09-e5637f916261?delete=true")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let check = sqlx::query_as::<_, (bool,)>(r#"SELECT enabled FROM checks WHERE uuid = "dd9a531a-1b0b-4a12-bc09-e5637f916261""#)
      .fetch_one(&*pool)
      .await;

    assert!(matches!(&check, Err(_)));

    pool.cleanup().await;

    Ok(())
  }
}
