use std::sync::Arc;

use anyhow::Context;
use chrono::Duration;
use jsonwebtoken::{DecodingKey, Validation};
use rocket::{
  serde::json::{Error as JsonError, Json},
  State,
};
use sqlx::{MySql, Pool};

use crate::{
  api::{
    auth::{Auth, Claims, RefreshToken, Tokens},
    error::Shortable,
    types::NewPassword,
    ApiResponse,
  },
  config::Config,
  model::User,
};

use super::error::{check_json, AppError};

#[derive(Serialize, Deserialize)]
pub struct Credentials {
  pub email: String,
  pub password: String,
}

#[post("/api/-/token", data = "<payload>")]
pub async fn token(config: &State<Arc<Config>>, pool: &State<Pool<MySql>>, payload: Json<Credentials>) -> ApiResponse<Json<Tokens>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let user = User::by_email(&mut *conn, &payload.email).await.context(AppError::InvalidCredentials).short()?;

  user.check_password(&payload.password).await.context(AppError::InvalidCredentials).short()?;

  Ok(Json(Tokens::generate(config.as_ref(), &user.uuid, Duration::hours(1)).context(AppError::ServerError).short()?))
}

#[post("/api/-/refresh", data = "<refresh>")]
pub async fn refresh(config: &State<Arc<Config>>, refresh: Json<RefreshToken>, pool: &State<Pool<MySql>>) -> ApiResponse<Json<Tokens>> {
  let claims = jsonwebtoken::decode::<Claims>(&refresh.refresh_token, &DecodingKey::from_secret(config.api.jwt_signing_key.as_ref()), &Validation::default())
    .context(AppError::InvalidCredentials)
    .short()?;

  if claims.claims.aud != "urn:defcon:refresh" {
    Err(anyhow!("invalid audience")).context(AppError::InvalidCredentials).short()?;
  }

  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let user = User::by_uuid(&mut *conn, &claims.claims.sub).await.context(AppError::InvalidCredentials).short()?;

  Ok(Json(Tokens::generate(config.as_ref(), &user.uuid, Duration::hours(1)).context(AppError::ServerError).short()?))
}

#[get("/api/-/me")]
pub async fn userinfo(auth: Auth, pool: &State<Pool<MySql>>) -> ApiResponse<Json<User>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let user = User::by_uuid(&mut *conn, &auth.sub).await.context("could not retrieve user").short()?;

  Ok(Json(user))
}

#[post("/api/-/password", data = "<payload>")]
pub async fn password(auth: Auth, pool: &State<Pool<MySql>>, payload: Result<Json<NewPassword>, JsonError<'_>>) -> ApiResponse<()> {
  let passwords = check_json(payload).short()?.0;
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;
  let user = User::by_uuid(&mut *conn, &auth.sub).await.short()?;

  user.check_password(&passwords.password).await.context(AppError::InvalidCredentials).short()?;
  user.update_password(&mut *conn, &passwords.new_password).await.context("could not change password").short()?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use chrono::{Duration, Utc};
  use jsonwebtoken::{self, EncodingKey, Header as JwtHeader};
  use rocket::{
    http::{Header, Status},
    serde::json::json,
  };

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

    let response = client.post("/api/-/token").body(body.to_string()).dispatch().await;
    assert_eq!(response.status(), Status::Ok);

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

    let response = client.post("/api/-/token").body(body.to_string()).dispatch().await;
    assert_eq!(response.status(), Status::Unauthorized);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn unauthorized_without_token() -> Result<()> {
    let (pool, client) = tests::authenticated_api_client().await.unwrap();

    pool.create_user().await?;

    let response = client.get("/api/checks").dispatch().await;
    assert_eq!(response.status(), Status::Unauthorized);

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn unauthorized_with_invalid_token() -> Result<()> {
    let (pool, client) = tests::authenticated_api_client().await.unwrap();

    pool.create_user().await?;

    let response = client.get("/api/checks").header(Header::new("authorization", "Bearer invalid")).dispatch().await;
    assert_eq!(response.status(), Status::Unauthorized);

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

    let response = client.get("/api/checks").header(Header::new("authorization", format!("Bearer {token}"))).dispatch().await;
    assert_eq!(response.status(), Status::Unauthorized);

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

    let response = client.get("/api/checks").header(Header::new("authorization", format!("Bearer {token}"))).dispatch().await;
    assert_eq!(response.status(), Status::Unauthorized);

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

    let response = client.get("/api/checks").header(Header::new("authorization", format!("Bearer {token}"))).dispatch().await;
    assert_eq!(response.status(), Status::Ok);

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

    let response = client.post("/api/-/refresh").body(format!(r#"{{ "refresh_token": "{token}" }}"#)).dispatch().await;
    assert_eq!(response.status(), Status::Ok);

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

    let response = client.post("/api/-/refresh").body(format!(r#"{{ "refresh_token": "{token}" }}"#)).dispatch().await;
    assert_eq!(response.status(), Status::Unauthorized);

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

    let response = client.post("/api/-/refresh").body(format!(r#"{{ "refresh_token": "{token}" }}"#)).dispatch().await;
    assert_eq!(response.status(), Status::Unauthorized);

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
        .post("/api/-/password")
        .header(Header::new("authorization", format!("Bearer {token}")))
        .body(payload.to_string())
        .dispatch()
        .await;

      assert_eq!(response.status(), Status::Ok);

      let user = User::by_email(&mut *conn, "noreply@example.com").await?;

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
        .post("/api/-/password")
        .header(Header::new("authorization", format!("Bearer {token}")))
        .body(payload.to_string())
        .dispatch()
        .await;

      assert_eq!(response.status(), Status::Unauthorized);

      let user = User::by_email(&mut *conn, "noreply@example.com").await?;

      assert!(matches!(user.check_password("password").await, Ok(())));
    }

    pool.cleanup().await;

    Ok(())
  }
}
