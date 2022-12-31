use std::{net::SocketAddr, sync::Arc};

use anyhow::{Context, Result};
use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::IntoResponse,
  routing::get,
  Router,
};
use kvlogger::*;
use sqlx::{MySql, Pool};

use defcon::{
  api::error::{AppError, ErrorResponse, Shortable},
  config::Config,
  model::{Check, DeadManSwitchLog},
};

pub async fn run(pool: Pool<MySql>, config: Arc<Config>) -> Result<()> {
  kvlog!(Info, "starting dead man switch process", {
    "listen" => config.dms.listen
  });

  let addr = SocketAddr::from((config.dms.listen.ip(), config.dms.listen.port()));
  let app = server(pool);

  axum::Server::bind(&addr).serve(app.into_make_service()).await.context("could not launch api process")?;

  Ok(())
}

fn server(pool: Pool<MySql>) -> Router {
  Router::new().route("/checkin/:uuid", get(checkin)).with_state(pool)
}

async fn checkin(pool: State<Pool<MySql>>, Path(uuid): Path<String>) -> Result<impl IntoResponse, ErrorResponse> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").context(AppError::ResourceNotFound).short()?;
  let check = Check::by_uuid(&mut conn, &uuid).await.context(AppError::ResourceNotFound).short()?;

  DeadManSwitchLog::insert(&mut conn, check.id).await.context(AppError::ServerError).short()?;

  Ok(StatusCode::OK)
}
