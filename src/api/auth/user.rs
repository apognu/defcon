use std::sync::Arc;

use anyhow::{Error, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use rocket::{
  http::Status,
  outcome::Outcome,
  request::{self, FromRequest, Request},
  State,
};
use sqlx::{MySql, Pool};

use crate::{api::error::AppError, config::Config, model::User};

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

#[async_trait]
impl<'r> FromRequest<'r> for Auth {
  type Error = Error;

  async fn from_request(request: &'r Request<'_>) -> request::Outcome<Auth, Error> {
    #[allow(unused_variables)]
    if let Outcome::Success(config) = request.guard::<&State<Arc<Config>>>().await {
      if config.api.skip_authentication {
        return Outcome::Success(Auth {
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

      let headers: Vec<_> = request.headers().get("authorization").collect();
      let token = headers.first().and_then(|value| value.strip_prefix("Bearer "));

      if let Some(token) = token {
        if let Ok(claims) = jsonwebtoken::decode::<Claims>(token, &DecodingKey::from_secret(config.api.jwt_signing_key.as_ref()), &Validation::default()) {
          if claims.claims.aud == "urn:defcon:access" {
            if let Outcome::Success(pool) = request.guard::<&State<Pool<MySql>>>().await {
              if let Ok(mut conn) = pool.acquire().await {
                if let Ok(user) = User::by_uuid(&mut conn, &claims.claims.sub).await {
                  return Outcome::Success(Auth { user });
                }
              }
            }
          }
        }
      }

      return Outcome::Failure((Status::Unauthorized, anyhow!("credentials could not be validated").context(AppError::InvalidCredentials)));
    }

    Outcome::Failure((Status::InternalServerError, anyhow!("could not retrieve config")))
  }
}
