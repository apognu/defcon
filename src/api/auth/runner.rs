use anyhow::{Context, Error, Result};
use axum::{
  extract::{FromRef, FromRequestParts},
  http::request::Parts,
  RequestPartsExt,
};
use axum_extra::{
  headers::{authorization::Bearer, Authorization},
  TypedHeader,
};
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation};

use crate::api::{
  error::{AppError, ErrorResponse, Shortable},
  AppState,
};

#[derive(Default, Clone)]
pub struct Keys {
  private: Option<EncodingKey>,
  public: Option<DecodingKey>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct RunnerClaims {
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

  pub fn generate(&self, claims: &RunnerClaims) -> Result<Option<String>> {
    match &self.private {
      None => Ok(None),
      Some(key) => {
        let now = Utc::now();

        let claims = RunnerClaims {
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

  pub fn verify(&self, token: Option<&str>) -> Result<TokenData<RunnerClaims>, Error> {
    match &self.public {
      None => Err(anyhow!("no public key found for runners").context(AppError::ServerError)),
      Some(key) => Ok(jsonwebtoken::decode::<RunnerClaims>(token.unwrap_or_default(), key, &Validation::new(Algorithm::ES256))?),
    }
  }
}

pub struct RunnerAuth {
  pub site: String,
}

impl<S> FromRequestParts<S> for RunnerAuth
where
  AppState: FromRef<S>,
  S: Send + Sync,
{
  type Rejection = ErrorResponse;

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let TypedHeader(Authorization(bearer)) = parts.extract::<TypedHeader<Authorization<Bearer>>>().await.context(AppError::InvalidCredentials).short()?;
    let state = AppState::from_ref(state);
    let keys = state.keys.unwrap();

    if let Ok(payload) = keys.verify(Some(bearer.token())) {
      if payload.claims.site.chars().all(|c| c == '-' || char::is_alphanumeric(c)) {
        return Ok(RunnerAuth { site: payload.claims.site });
      }
    }

    Err(anyhow!("cannot authenticate the request").context(AppError::InvalidCredentials)).short()
  }
}
