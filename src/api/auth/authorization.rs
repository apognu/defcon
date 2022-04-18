use anyhow::{Context, Error, Result};
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation};
use regex::Regex;
use rocket::{
  http::Status,
  outcome::Outcome,
  request::{self, FromRequest, Request},
  State,
};

use crate::api::error::AppError;

#[derive(Default, Clone)]
pub struct Keys {
  private: Option<EncodingKey>,
  public: Option<DecodingKey>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Claims {
  pub iat: u64,
  pub exp: u64,
  pub site: String,
}

impl Keys {
  pub fn new_private(pem: &'static [u8]) -> Result<Keys> {
    let keys = Keys {
      private: Some(EncodingKey::from_ec_pem(pem).context("invalid private key format")?),
      ..Default::default()
    };

    Ok(keys)
  }

  pub fn new_public(pem: &'static [u8]) -> Result<Keys> {
    let keys = Keys {
      public: Some(DecodingKey::from_ec_pem(pem).context("invalid public key format")?),
      ..Default::default()
    };

    Ok(keys)
  }

  pub fn generate(&self, claims: &Claims) -> Result<Option<String>> {
    match &self.private {
      None => Ok(None),
      Some(key) => {
        let now = Utc::now();

        let claims = Claims {
          iat: now.timestamp() as u64,
          exp: now.timestamp() as u64 + 30,
          ..claims.to_owned()
        };

        let header = Header {
          alg: Algorithm::ES256,
          ..Default::default()
        };

        Ok(Some(jsonwebtoken::encode(&header, &claims, key)?))
      }
    }
  }

  pub fn verify(&self, token: Option<&str>) -> Result<TokenData<Claims>, Error> {
    match &self.public {
      None => Err(anyhow!("no public key found for runners").context(AppError::ServerError)),
      Some(key) => Ok(jsonwebtoken::decode::<Claims>(token.unwrap_or_default(), key, &Validation::new(Algorithm::ES256))?),
    }
  }
}

pub struct RunnerCredentials {
  pub site: String,
}

#[async_trait]
impl<'r> FromRequest<'r> for RunnerCredentials {
  type Error = Error;

  async fn from_request(request: &'r Request<'_>) -> request::Outcome<RunnerCredentials, Error> {
    let headers: Vec<_> = request.headers().get("authorization").collect();
    let token = headers.get(0).and_then(|value| value.strip_prefix("Bearer "));
    let rgx = Regex::new(r"^[a-z0-9-]+$").unwrap();

    if let Outcome::Success(guard) = request.guard::<&State<Keys>>().await {
      if let Ok(payload) = guard.verify(token) {
        if rgx.is_match(&payload.claims.site) {
          return Outcome::Success(RunnerCredentials { site: payload.claims.site });
        }
      }
    }

    Outcome::Failure((Status::Unauthorized, anyhow!("credentials could not be validated").context(AppError::InvalidCredentials)))
  }
}
