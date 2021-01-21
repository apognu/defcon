use std::fmt::Display;

use anyhow::Error;
use rocket::{
  http::Status,
  request::Request,
  response::{self, status::Custom, Responder},
};
use rocket_contrib::json::{Json, JsonError};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
  #[error("bad request: {0}")]
  BadRequest(anyhow::Error),
  #[error("resource not found: {0}")]
  ResourceNotFound(anyhow::Error),
  #[error("server error")]
  ServerError(anyhow::Error),
}

impl AppError {
  fn code(&self) -> u64 {
    match self {
      AppError::BadRequest(_) => 400,
      AppError::ResourceNotFound(_) => 404,
      AppError::ServerError(_) => 500,
    }
  }
}

#[derive(Debug, Serialize)]
pub struct ApiError {
  code: u64,
  message: String,
}

pub fn check_json<T>(payload: Result<T, JsonError>) -> Result<T, AppError> {
  match payload {
    Ok(payload) => Ok(payload),
    Err(err) => match err {
      JsonError::Io(err) => Err(AppError::BadRequest(anyhow!(err))),
      JsonError::Parse(_, err) => Err(AppError::BadRequest(anyhow!(err))),
    },
  }
}

pub fn server_error<E>(err: E) -> AppError
where
  E: Into<Error> + Display,
{
  let err = err.into();

  crate::log_error(&err);

  AppError::ServerError(err)
}

impl ApiError {
  pub fn new<S>(code: u64, message: S) -> ApiError
  where
    S: Into<String>,
  {
    ApiError { code, message: message.into() }
  }
}

impl From<AppError> for ApiError {
  fn from(error: AppError) -> ApiError {
    let code = error.code();

    ApiError { code, message: error.to_string() }
  }
}

impl From<&AppError> for ApiError {
  fn from(error: &AppError) -> ApiError {
    let code = error.code();

    ApiError { code, message: error.to_string() }
  }
}

impl<'r> Responder<'r, 'static> for ApiError {
  fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
    Json(self).respond_to(request)
  }
}

pub trait Errorable<'a, T> {
  fn apierr(self) -> Result<T, Custom<Option<ApiError>>>;
}

impl<'a, T> Errorable<'a, T> for Result<T, Error>
where
  T: std::fmt::Debug,
{
  fn apierr(self) -> Result<T, Custom<Option<ApiError>>> {
    self.map_err(|err| match err.downcast_ref() {
      Some(inner @ AppError::BadRequest(_)) => Custom(Status::BadRequest, Some(inner.into())),
      Some(inner @ AppError::ResourceNotFound(_)) => Custom(Status::NotFound, Some(inner.into())),
      Some(inner @ AppError::ServerError(_)) => Custom(Status::InternalServerError, Some(inner.into())),
      None => Custom(Status::InternalServerError, Some(ApiError::new(500, err.to_string()))),
    })
  }
}

impl<'a, T> Errorable<'a, T> for Result<T, AppError> {
  fn apierr(self) -> Result<T, Custom<Option<ApiError>>> {
    self.map_err(|err| match err {
      inner @ AppError::BadRequest(_) => Custom(Status::BadRequest, Some(inner.into())),
      inner @ AppError::ResourceNotFound(_) => Custom(Status::NotFound, Some(inner.into())),
      inner @ AppError::ServerError(_) => Custom(Status::InternalServerError, Some(inner.into())),
    })
  }
}
