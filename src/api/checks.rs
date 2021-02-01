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
    name: payload.check.name,
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

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use rocket::http::Status;
  use rocket_contrib::json;

  use crate::{api::types as api, spec};

  #[tokio::test]
  async fn list_checks() -> Result<()> {
    let (pool, client) = spec::api_client().await?;

    let enabled = json!({
      "name": "list_checks()",
      "enabled": true,
      "interval": "10s",
      "passing_threshold": 1,
      "failing_threshold": 1,
      "spec": {
        "kind": "app_store",
        "bundle_id": "helloworld"
      }
    });

    let disabled = json!({
      "name": "list_checks()",
      "enabled": false,
      "interval": "10s",
      "passing_threshold": 1,
      "failing_threshold": 1,
      "spec": {
        "kind": "app_store",
        "bundle_id": "helloworld"
      }
    });

    client.post("/api/checks").body(enabled.to_string().as_bytes()).dispatch().await;
    client.post("/api/checks").body(disabled.to_string().as_bytes()).dispatch().await;

    let checks = sqlx::query_as::<_, (String,)>("SELECT name FROM checks WHERE enabled = 1").fetch_all(&*pool).await?;

    assert_eq!(checks.len(), 1);
    assert_eq!(&checks[0].0, "list_checks()");

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn list_checks_all() -> Result<()> {
    let (pool, client) = spec::api_client().await?;

    let enabled = json!({
      "name": "list_checks_all()",
      "enabled": true,
      "interval": "10s",
      "passing_threshold": 1,
      "failing_threshold": 1,
      "spec": {
        "kind": "app_store",
        "bundle_id": "helloworld"
      }
    });

    let disabled = json!({
      "name": "list_checks_all()",
      "enabled": false,
      "interval": "10s",
      "passing_threshold": 1,
      "failing_threshold": 1,
      "spec": {
        "kind": "app_store",
        "bundle_id": "helloworld"
      }
    });

    client.post("/api/checks").body(enabled.to_string().as_bytes()).dispatch().await;
    client.post("/api/checks").body(disabled.to_string().as_bytes()).dispatch().await;

    let checks = sqlx::query_as::<_, (String,)>("SELECT name FROM checks").fetch_all(&*pool).await?;

    assert_eq!(checks.len(), 2);
    assert_eq!(&checks[0].0, "list_checks_all()");

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn get_check() -> Result<()> {
    let (pool, client) = spec::api_client().await?;

    sqlx::query(
      r#"INSERT INTO checks (id, uuid, name, kind, enabled, `interval`, passing_threshold, failing_threshold) VALUES ( 1, "dd9a531a-1b0b-4a12-bc09-e5637f916261", "get_check()", "tcp", 0, 10, 1, 1 )"#,
    )
    .execute(&*pool)
    .await?;

    sqlx::query(r#"INSERT INTO tcp_specs (check_id, host, port, timeout) VALUES ( 1, "0.0.0.0", 0, 0 )"#)
      .execute(&*pool)
      .await?;

    let response = client.get("/api/checks/dd9a531a-1b0b-4a12-bc09-e5637f916261").dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    let check: api::Check = serde_json::from_str(&response.into_string().await.unwrap())?;
    assert_eq!(&check.check.name, "get_check()");

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn get_check_not_found() -> Result<()> {
    let (pool, client) = spec::api_client().await?;

    let response = client.get("/api/checks/nonexisting").dispatch().await;
    assert_eq!(response.status(), Status::NotFound);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn create() -> Result<()> {
    let (pool, client) = spec::api_client().await?;

    let check = json!({
      "name": "create()",
      "enabled": false,
      "interval": "10s",
      "passing_threshold": 1,
      "failing_threshold": 1,
      "spec": {
        "kind": "app_store",
        "bundle_id": "helloworld"
      }
    });

    let response = client.post("/api/checks").body(check.to_string().as_bytes()).dispatch().await;
    assert_eq!(response.status(), Status::Created);

    let checks = sqlx::query_as::<_, (String,)>("SELECT name FROM checks").fetch_all(&*pool).await?;

    assert_eq!(checks.len(), 1);
    assert_eq!(&checks[0].0, "create()");

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn create_bad_request() -> Result<()> {
    let (pool, client) = spec::api_client().await?;

    let check = json!({
      "name": "create_bad_request()",
      "enabled": false,
      "interval": "10s",
      "passing_threshold": 1,
      "failing_threshold": 1
    });

    let response = client.post("/api/checks").body(check.to_string().as_bytes()).dispatch().await;
    assert_eq!(response.status(), Status::BadRequest);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn update() -> Result<()> {
    let (pool, client) = spec::api_client().await?;

    sqlx::query(
      r#"INSERT INTO checks (id, uuid, name, kind, enabled, `interval`, passing_threshold, failing_threshold) VALUES ( 1, "dd9a531a-1b0b-4a12-bc09-e5637f916261", "update()", "tcp", 0, 10, 1, 1 )"#,
    )
    .execute(&*pool)
    .await?;

    sqlx::query(r#"INSERT INTO tcp_specs (check_id, host, port, timeout) VALUES ( 1, "0.0.0.0", 80, 10 )"#)
      .execute(&*pool)
      .await?;

    let check = json!({
      "name": "new_update()",
      "enabled": false,
      "interval": "15s",
      "passing_threshold": 1,
      "failing_threshold": 1,
      "spec": {
        "kind": "tcp",
        "host": "1.2.3.4",
        "port": 81,
        "timeout": "1h"
      }
    });

    let response = client.put("/api/checks/dd9a531a-1b0b-4a12-bc09-e5637f916261").body(check.to_string().as_bytes()).dispatch().await;
    assert_eq!(response.status(), Status::Ok);

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
  async fn patch() -> Result<()> {
    let (pool, client) = spec::api_client().await?;

    sqlx::query(
      r#"INSERT INTO checks (id, uuid, name, kind, enabled, `interval`, passing_threshold, failing_threshold) VALUES ( 1, "dd9a531a-1b0b-4a12-bc09-e5637f916261", "patch()", "tcp", 0, 10, 1, 1 )"#,
    )
    .execute(&*pool)
    .await?;

    sqlx::query(r#"INSERT INTO tcp_specs (check_id, host, port, timeout) VALUES ( 1, "0.0.0.0", 80, 10 )"#)
      .execute(&*pool)
      .await?;

    let check = json!({
      "name": "new_update()",
      "interval": "10m"
    });

    let response = client.patch("/api/checks/dd9a531a-1b0b-4a12-bc09-e5637f916261").body(check.to_string().as_bytes()).dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    let check = sqlx::query_as::<_, (String, bool, u64)>(r#"SELECT name, enabled, `interval` FROM checks WHERE uuid = "dd9a531a-1b0b-4a12-bc09-e5637f916261""#)
      .fetch_one(&*pool)
      .await?;

    let spec = sqlx::query_as::<_, (String, u16, u64)>(r#"SELECT host, port, timeout FROM tcp_specs WHERE check_id = 1"#)
      .fetch_one(&*pool)
      .await?;

    assert_eq!(&check.0, "new_update()");
    assert_eq!(check.1, false);
    assert_eq!(check.2, 600);
    assert_eq!(spec.0, "0.0.0.0");
    assert_eq!(spec.1, 80);
    assert_eq!(spec.2, 10);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn delete() -> Result<()> {
    let (pool, client) = spec::api_client().await?;

    sqlx::query(
      r#"INSERT INTO checks (id, uuid, name, kind, enabled, `interval`, passing_threshold, failing_threshold) VALUES ( 1, "dd9a531a-1b0b-4a12-bc09-e5637f916261", "patch()", "tcp", 1, 10, 1, 1 )"#,
    )
    .execute(&*pool)
    .await?;

    let response = client.delete("/api/checks/dd9a531a-1b0b-4a12-bc09-e5637f916261").dispatch().await;
    assert_eq!(response.status(), Status::NoContent);

    let check = sqlx::query_as::<_, (bool,)>(r#"SELECT enabled FROM checks WHERE uuid = "dd9a531a-1b0b-4a12-bc09-e5637f916261""#)
      .fetch_one(&*pool)
      .await?;

    assert_eq!(check.0, false);

    pool.cleanup().await;

    Ok(())
  }
}
