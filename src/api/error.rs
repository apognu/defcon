use anyhow::{Context, Error};
use rocket::{
  http::Status,
  request::Request,
  response::{self, status::Custom, Responder, Response},
};
use rocket_contrib::{json, json::JsonError};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
  #[error("bad request")]
  BadRequest,
  #[error("invalid credentials")]
  InvalidCredentials,
  #[error("missing resource")]
  ResourceNotFound,
  #[error("server error, please check your logs for more information")]
  ServerError,
  #[error("database error, please check your logs for more information")]
  DatabaseError,
}

pub struct ErrorResponse(pub Custom<anyhow::Error>);

impl ErrorResponse {
  fn status(&self) -> &'static str {
    match self.code().code {
      400 => "bad_request",
      404 => "not_found",
      500 => "server_error",
      _ => "unknown_error",
    }
  }

  fn code(&self) -> Status {
    self.0 .0
  }

  fn error(self) -> Error {
    self.0 .1
  }
}

pub fn check_json<T>(payload: Result<T, JsonError>) -> Result<T, Error> {
  match payload {
    Ok(payload) => Ok(payload),

    Err(err) => match err {
      JsonError::Io(err) => Err(err).context(AppError::BadRequest),
      JsonError::Parse(_, err) => Err(err).context(AppError::BadRequest),
    },
  }
}

impl<'r> Responder<'r, 'static> for ErrorResponse {
  fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
    let code = self.code();
    let status = self.status();
    let error = self.error();
    let details = error.chain().skip(1).map(ToString::to_string).collect::<Vec<_>>().join(": ");

    let payload = json!({
      "status": status,
      "message": error.to_string(),
      "details": details
    });

    Response::build_from(payload.respond_to(request)?).status(code).ok()
  }
}

pub trait Shortable<'a, T> {
  type Output;

  fn short(self) -> Self::Output;
}

impl<'a, T> Shortable<'a, T> for Result<T, Error> {
  type Output = Result<T, ErrorResponse>;

  fn short(self) -> Self::Output {
    self.map_err(|err| match err.downcast_ref::<AppError>() {
      Some(AppError::BadRequest) => ErrorResponse(Custom(Status::BadRequest, err)),
      Some(AppError::InvalidCredentials) => ErrorResponse(Custom(Status::Unauthorized, err)),
      Some(AppError::ResourceNotFound) => ErrorResponse(Custom(Status::NotFound, err)),
      Some(AppError::ServerError) => ErrorResponse(Custom(Status::InternalServerError, err)),
      Some(AppError::DatabaseError) => ErrorResponse(Custom(Status::InternalServerError, err)),
      None => ErrorResponse(Custom(Status::InternalServerError, err)),
    })
  }
}

impl<'a, T> Shortable<'a, T> for Result<T, sqlx::Error> {
  type Output = Result<T, Error>;

  fn short(self) -> Self::Output {
    self.map_err(|err| match err {
      sqlx::Error::RowNotFound => anyhow!(err).context(AppError::ResourceNotFound),
      err => anyhow!(err).context(AppError::DatabaseError),
    })
  }
}
