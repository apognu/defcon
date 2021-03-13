use chrono::Local;
use kvlogger::*;
use rocket::{
  fairing::{Fairing, Info, Kind},
  request::Request,
  response::Response,
};

#[derive(Default)]
pub struct ApiLogger;

impl ApiLogger {
  pub fn new() -> ApiLogger {
    ApiLogger
  }
}

#[async_trait]
impl Fairing for ApiLogger {
  fn info(&self) -> Info {
    Info {
      name: "Logging middleware",
      kind: Kind::Request | Kind::Response,
    }
  }

  async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
    let ip = request.client_ip().map(|ip| ip.to_string()).unwrap_or_else(|| "-".to_string());
    let time = Local::now().format("%Y-%m-%dT%H:%M:%S%z");

    #[allow(clippy::branches_sharing_code)]
    if request.uri().path().starts_with("/api/runner/") {
      kvlog!(Debug, format!("{} {}", request.method(), request.uri()), {
        "time" => time,
        "remote" => ip,
        "method" => request.method(),
        "path" => request.uri(),
        "status" => response.status().code
      });
    } else {
      kvlog!(Info, format!("{} {}", request.method(), request.uri()), {
        "time" => time,
        "remote" => ip,
        "method" => request.method(),
        "path" => request.uri(),
        "status" => response.status().code
      });
    }
  }
}
