use std::net::SocketAddr;

use axum::{
  extract::ConnectInfo,
  http::{Request, StatusCode},
  middleware::Next,
  response::Response,
  RequestPartsExt,
};
use chrono::Local;

use kvlogger::kvlog;

pub async fn api_logger<B>(request: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
  let time = Local::now().format("%Y-%m-%dT%H:%M:%S%z");
  let method = request.method().clone();
  let uri = request.uri().clone();

  let (mut parts, body) = request.into_parts();
  let ip = if let Ok(ConnectInfo(addr)) = parts.extract::<ConnectInfo<SocketAddr>>().await {
    addr.ip().to_string()
  } else {
    "N/A".to_string()
  };

  let response = next.run(Request::from_parts(parts, body)).await;

  kvlog!(Info, format!("{} {}", method, uri), {
    "time" => time,
    "remote" => ip,
    "method" => method,
    "path" => uri,
    "status" => response.status().as_u16()
  });

  Ok(response)
}
