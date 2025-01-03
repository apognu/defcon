use anyhow::Error;
use axum::{
  extract::rejection::{JsonDataError, JsonRejection, JsonRejection::*},
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
  #[error("bad request")]
  BadRequest,
  #[error("bad request")]
  InvalidPayload(#[from] JsonDataError),
  #[error("invalid credentials")]
  InvalidCredentials,
  #[error("missing resource")]
  ResourceNotFound,
  #[error("resource already exists or cannot be created")]
  Conflict,
  #[error("server error, please check your logs for more information")]
  ServerError,
  #[error("database error, please check your logs for more information")]
  DatabaseError,
}

pub struct ErrorResponse(StatusCode, anyhow::Error);

impl ErrorResponse {
  fn code(&self) -> StatusCode {
    self.0
  }

  fn error(&self) -> &Error {
    &self.1
  }
}

impl IntoResponse for ErrorResponse {
  fn into_response(self) -> Response {
    let error = self.error();
    let details = error.chain().skip(1).map(ToString::to_string).collect::<Vec<_>>().join(": ");

    let payload = if details.is_empty() {
      json!({
        "message": error.to_string(),
      })
    } else {
      json!({
        "message": error.to_string(),
        "details": details
      })
    };

    (self.code(), Json(payload)).into_response()
  }
}

pub fn check_json<T>(payload: Result<Json<T>, JsonRejection>) -> Result<T, Error> {
  match payload {
    Ok(Json(payload)) => Ok(payload),
    Err(err) => match err {
      JsonDataError(err) => Err(anyhow!(AppError::InvalidPayload(err))),
      _ => Err(anyhow!(AppError::BadRequest)),
    },
  }
}

pub trait Shortable<'a, T> {
  type Output;

  fn short(self) -> Self::Output;
}

impl<T> Shortable<'_, T> for Result<T, Error> {
  type Output = Result<T, ErrorResponse>;

  fn short(self) -> Self::Output {
    self.map_err(|err| match err.downcast_ref::<AppError>() {
      Some(AppError::BadRequest) => ErrorResponse(StatusCode::BAD_REQUEST, err),
      Some(AppError::InvalidPayload(_)) => ErrorResponse(StatusCode::BAD_REQUEST, err),
      Some(AppError::InvalidCredentials) => ErrorResponse(StatusCode::UNAUTHORIZED, anyhow!("provided credentials are invalid")),
      Some(AppError::ResourceNotFound) => ErrorResponse(StatusCode::NOT_FOUND, err),
      Some(AppError::Conflict) => ErrorResponse(StatusCode::CONFLICT, err),
      _ => ErrorResponse(StatusCode::INTERNAL_SERVER_ERROR, err),
    })
  }
}

impl<T> Shortable<'_, T> for Result<T, sqlx::Error> {
  type Output = Result<T, Error>;

  fn short(self) -> Self::Output {
    self.map_err(|err| match err {
      sqlx::Error::RowNotFound => anyhow!(err).context(AppError::ResourceNotFound),
      err => anyhow!(err).context(AppError::DatabaseError),
    })
  }
}
