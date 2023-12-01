use std::sync::Arc;

use anyhow::Context;
use axum::{
  extract::{rejection::JsonRejection, State},
  Json,
};
use chrono::Duration;
use jsonwebtoken::{DecodingKey, Validation};
use sqlx::{MySql, Pool};

use crate::{
  api::{
    auth::{Auth, Claims, RefreshToken, Tokens},
    error::Shortable,
    types::{ApiKey, Credentials, NewPassword},
    ApiResponse,
  },
  config::Config,
  model::User,
};

use super::error::{check_json, AppError};

pub async fn token(State(config): State<Arc<Config>>, State(pool): State<Pool<MySql>>, Json(payload): Json<Credentials>) -> ApiResponse<Json<Tokens>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let user = User::by_email(&mut conn, &payload.email).await.context(AppError::InvalidCredentials).short()?;

  user.check_password(&payload.password).await.context(AppError::InvalidCredentials).short()?;

  Ok(Json(Tokens::generate(config.as_ref(), &user.uuid, Duration::hours(1)).context(AppError::ServerError).short()?))
}

pub async fn refresh(config: State<Arc<Config>>, pool: State<Pool<MySql>>, Json(payload): Json<RefreshToken>) -> ApiResponse<Json<Tokens>> {
  let mut validation = Validation::default();
  validation.set_audience(&["urn:defcon:refresh"]);

  let claims = jsonwebtoken::decode::<Claims>(&payload.refresh_token, &DecodingKey::from_secret(config.api.jwt_signing_key.as_ref()), &validation)
    .context(AppError::InvalidCredentials)
    .short()?;

  if claims.claims.aud != "urn:defcon:refresh" {
    Err(anyhow!("invalid audience")).context(AppError::InvalidCredentials).short()?;
  }

  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let user = User::by_uuid(&mut conn, &claims.claims.sub).await.context(AppError::InvalidCredentials).short()?;

  Ok(Json(Tokens::generate(config.as_ref(), &user.uuid, Duration::hours(1)).context(AppError::ServerError).short()?))
}

pub async fn userinfo(auth: Auth) -> ApiResponse<Json<User>> {
  Ok(Json(auth.user))
}

pub async fn password(auth: Auth, pool: State<Pool<MySql>>, payload: Result<Json<NewPassword>, JsonRejection>) -> ApiResponse<()> {
  let payload = check_json(payload).short()?;
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  auth.user.check_password(&payload.password).await.context(AppError::InvalidCredentials).short()?;
  auth.user.update_password(&mut conn, &payload.new_password).await.context("could not change password").short()?;

  Ok(())
}

pub async fn api_key(auth: Auth, pool: State<Pool<MySql>>) -> ApiResponse<Json<ApiKey>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let api_key = auth.user.generate_api_key(&mut conn).await.context("could not generate API key").short()?;

  Ok(Json(ApiKey { api_key }))
}

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use axum::{
    body::Body,
    http::{Request, StatusCode},
  };
  use chrono::{Duration, Utc};
  use hyper::Method;
  use jsonwebtoken::{self, EncodingKey, Header as JwtHeader};
  use serde_json::json;
  use tower::ServiceExt;

  use super::*;
  use crate::{
    api::auth::Claims,
    tests::{self, JWT_SIGNING_KEY},
  };

  #[tokio::test]
  async fn authorized_with_good_credentials() -> Result<()> {
    let (pool, client) = tests::authenticated_api_client().await.unwrap();

    pool.create_user().await?;

    let body = json!({
      "email": "noreply@example.com",
      "password": "password"
    });

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/-/token")
          .header("content-type", "application/json")
          .body(Body::from(body.to_string()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn unauthorized_with_invalid_credentials() -> Result<()> {
    let (pool, client) = tests::authenticated_api_client().await.unwrap();

    pool.create_user().await?;

    let body = json!({
      "email": "noreply@example.com",
      "password": "wrongpassword"
    });

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/-/token")
          .header("content-type", "application/json")
          .body(Body::from(body.to_string()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn unauthorized_without_token() -> Result<()> {
    let (pool, client) = tests::authenticated_api_client().await.unwrap();

    pool.create_user().await?;

    let response = client.oneshot(Request::builder().uri("/api/checks").body(Body::empty()).unwrap()).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn unauthorized_with_invalid_token() -> Result<()> {
    let (pool, client) = tests::authenticated_api_client().await.unwrap();

    pool.create_user().await?;

    let response = client
      .oneshot(Request::builder().uri("/api/checks").header("authorization", "Bearer invalid").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn unauthorized_with_token_invalid_key() -> Result<()> {
    let (pool, client) = tests::authenticated_api_client().await.unwrap();

    pool.create_user().await?;

    let now = Utc::now();
    let exp = now + Duration::hours(1);

    let claims = Claims {
      aud: "urn:defcon:access".to_string(),
      iat: now.timestamp(),
      exp: exp.timestamp(),
      sub: "7fc3989e-baea-4c7b-99a9-9210d2a3422c".to_string(),
    };

    let token = jsonwebtoken::encode(&JwtHeader::default(), &claims, &EncodingKey::from_secret("invalidkey".as_ref())).unwrap();

    let response = client
      .oneshot(Request::builder().uri("/api/checks").header("authorization", format!("Bearer {token}")).body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn unauthorized_with_expired_token() -> Result<()> {
    let (pool, client) = tests::authenticated_api_client().await.unwrap();

    pool.create_user().await?;

    let now = Utc::now() - Duration::hours(2);
    let exp = now - Duration::hours(1);

    let claims = Claims {
      aud: "urn:defcon:access".to_string(),
      iat: now.timestamp(),
      exp: exp.timestamp(),
      sub: "7fc3989e-baea-4c7b-99a9-9210d2a3422c".to_string(),
    };

    let token = jsonwebtoken::encode(&JwtHeader::default(), &claims, &EncodingKey::from_secret(JWT_SIGNING_KEY.as_ref())).unwrap();

    let response = client
      .oneshot(Request::builder().uri("/api/checks").header("authorization", format!("Bearer {token}")).body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn authorized_with_token() -> Result<()> {
    let (pool, client) = tests::authenticated_api_client().await.unwrap();

    pool.create_user().await?;

    let now = Utc::now();
    let exp = now + Duration::hours(1);

    let claims = Claims {
      aud: "urn:defcon:access".to_string(),
      iat: now.timestamp(),
      exp: exp.timestamp(),
      sub: "7fc3989e-baea-4c7b-99a9-9210d2a3422c".to_string(),
    };

    let token = jsonwebtoken::encode(&JwtHeader::default(), &claims, &EncodingKey::from_secret(JWT_SIGNING_KEY.as_ref())).unwrap();

    println!("{token}");

    let response = client
      .oneshot(Request::builder().uri("/api/checks").header("authorization", format!("Bearer {token}")).body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn refreshes_with_token() -> Result<()> {
    let (pool, client) = tests::authenticated_api_client().await.unwrap();

    pool.create_user().await?;

    let now = Utc::now();
    let exp = now + Duration::hours(1);

    let claims = Claims {
      aud: "urn:defcon:refresh".to_string(),
      iat: now.timestamp(),
      exp: exp.timestamp(),
      sub: "7fc3989e-baea-4c7b-99a9-9210d2a3422c".to_string(),
    };

    let token = jsonwebtoken::encode(&JwtHeader::default(), &claims, &EncodingKey::from_secret(JWT_SIGNING_KEY.as_ref())).unwrap();

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/-/refresh")
          .header("content-type", "application/json")
          .body(Body::from(format!(r#"{{ "refresh_token": "{token}" }}"#)))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn fails_refresh_with_invalid_token() -> Result<()> {
    let (pool, client) = tests::authenticated_api_client().await.unwrap();

    pool.create_user().await?;

    let now = Utc::now();
    let exp = now + Duration::hours(1);

    let claims = Claims {
      aud: "urn:defcon:refresh".to_string(),
      iat: now.timestamp(),
      exp: exp.timestamp(),
      sub: "7fc3989e-baea-4c7b-99a9-9210d2a3422c".to_string(),
    };

    let token = jsonwebtoken::encode(&JwtHeader::default(), &claims, &EncodingKey::from_secret("invalidkey".as_ref())).unwrap();

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/-/refresh")
          .header("content-type", "application/json")
          .body(Body::from(format!(r#"{{ "refresh_token": "{token}" }}"#)))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn fails_refresh_with_invalid_audience() -> Result<()> {
    let (pool, client) = tests::authenticated_api_client().await.unwrap();

    pool.create_user().await?;

    let now = Utc::now();
    let exp = now + Duration::hours(1);

    let claims = Claims {
      aud: "urn:defcon:access".to_string(),
      iat: now.timestamp(),
      exp: exp.timestamp(),
      sub: "7fc3989e-baea-4c7b-99a9-9210d2a3422c".to_string(),
    };

    let token = jsonwebtoken::encode(&JwtHeader::default(), &claims, &EncodingKey::from_secret(JWT_SIGNING_KEY.as_ref())).unwrap();

    let response = client
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/-/refresh")
          .header("content-type", "application/json")
          .body(Body::from(format!(r#"{{ "refresh_token": "{token}" }}"#)))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn change_password_succeeds() -> Result<()> {
    let (pool, client) = tests::authenticated_api_client().await.unwrap();

    pool.create_user().await?;

    {
      let mut conn = pool.acquire().await?;

      let now = Utc::now();
      let exp = now + Duration::hours(1);

      let claims = Claims {
        aud: "urn:defcon:access".to_string(),
        iat: now.timestamp(),
        exp: exp.timestamp(),
        sub: "7fc3989e-baea-4c7b-99a9-9210d2a3422c".to_string(),
      };

      let token = jsonwebtoken::encode(&JwtHeader::default(), &claims, &EncodingKey::from_secret(JWT_SIGNING_KEY.as_ref())).unwrap();

      let payload = json!({
        "password": "password",
        "new_password": "newpassword",
      });

      let response = client
        .oneshot(
          Request::builder()
            .method(Method::POST)
            .uri("/api/-/password")
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap(),
        )
        .await
        .unwrap();

      assert_eq!(response.status(), StatusCode::OK);

      let user = User::by_email(&mut conn, "noreply@example.com").await?;

      assert!(matches!(user.check_password("newpassword").await, Ok(())));
    }

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn change_password_fails() -> Result<()> {
    let (pool, client) = tests::authenticated_api_client().await.unwrap();

    pool.create_user().await?;

    {
      let mut conn = pool.acquire().await?;

      let now = Utc::now();
      let exp = now + Duration::hours(1);

      let claims = Claims {
        aud: "urn:defcon:access".to_string(),
        iat: now.timestamp(),
        exp: exp.timestamp(),
        sub: "7fc3989e-baea-4c7b-99a9-9210d2a3422c".to_string(),
      };

      let token = jsonwebtoken::encode(&JwtHeader::default(), &claims, &EncodingKey::from_secret(JWT_SIGNING_KEY.as_ref())).unwrap();

      let payload = json!({
        "password": "wrongpassword",
        "new_password": "newpassword",
      });

      let response = client
        .oneshot(
          Request::builder()
            .method(Method::POST)
            .uri("/api/-/password")
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap(),
        )
        .await
        .unwrap();

      assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

      let user = User::by_email(&mut conn, "noreply@example.com").await?;

      assert!(matches!(user.check_password("password").await, Ok(())));
    }

    pool.cleanup().await;

    Ok(())
  }
}
