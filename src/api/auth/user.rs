use anyhow::{Context, Result};
use axum::{
  extract::{FromRef, FromRequestParts},
  http::request::Parts,
  RequestPartsExt,
};
use axum_extra::{
  headers::{authorization::Bearer, Authorization},
  TypedHeader,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};

use crate::{
  api::{
    error::{AppError, ErrorResponse, Shortable},
    AppState,
  },
  config::Config,
  model::User,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub aud: String,
  pub iat: i64,
  pub exp: i64,
  pub sub: String,
}

#[derive(Serialize)]
pub struct Tokens {
  pub access_token: String,
  pub refresh_token: String,
}

impl Tokens {
  pub fn generate(config: &Config, sub: &str, duration: Duration) -> Result<Tokens> {
    let now = Utc::now();
    let exp = now + duration;

    let claims = Claims {
      aud: "urn:defcon:access".to_string(),
      iat: now.timestamp(),
      exp: exp.timestamp(),
      sub: sub.to_string(),
    };

    let access_token = jsonwebtoken::encode(&Header::default(), &claims, &EncodingKey::from_secret(config.api.jwt_signing_key.as_ref()))?;

    let exp = now + duration + Duration::hours(72);

    let claims = Claims {
      aud: "urn:defcon:refresh".to_string(),
      iat: now.timestamp(),
      exp: exp.timestamp(),
      sub: sub.to_string(),
    };

    let refresh_token = jsonwebtoken::encode(&Header::default(), &claims, &EncodingKey::from_secret(config.api.jwt_signing_key.as_ref()))?;

    Ok(Tokens { access_token, refresh_token })
  }
}

pub struct Auth {
  pub user: User,
}

pub struct RefreshAuth {
  pub sub: String,
}

#[derive(Deserialize)]
pub struct RefreshToken {
  pub refresh_token: String,
}

impl<S> FromRequestParts<S> for Auth
where
  AppState: FromRef<S>,
  S: Send + Sync,
{
  type Rejection = ErrorResponse;

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let state = AppState::from_ref(state);
    let config = state.config;
    let pool = state.pool;

    if config.api.skip_authentication {
      return Ok(Auth {
        user: User {
          id: 0,
          uuid: "7fc3989e-baea-4c7b-99a9-9210d2a3422c".to_string(),
          email: "noreply@example.com".to_string(),
          password: "".to_string(),
          name: "".to_string(),
          api_key: None,
        },
      });
    }

    let TypedHeader(Authorization(bearer)) = parts.extract::<TypedHeader<Authorization<Bearer>>>().await.context(AppError::InvalidCredentials).short()?;
    let secret = DecodingKey::from_secret(config.api.jwt_signing_key.as_ref());

    let mut validation = Validation::default();
    validation.set_audience(&["urn:defcon:access"]);

    if let Ok(claims) = jsonwebtoken::decode::<Claims>(bearer.token(), &secret, &validation) {
      if let Ok(mut conn) = pool.acquire().await {
        if let Ok(user) = User::by_uuid(&mut conn, &claims.claims.sub).await {
          return Ok(Auth { user });
        }
      }
    }

    Err(anyhow!(AppError::InvalidCredentials)).short()
  }
}
