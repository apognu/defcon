use std::{sync::Arc, time::Instant};

use anyhow::{Context, Result};
use async_trait::async_trait;
use sha2::{Digest, Sha512};
use sqlx::MySqlConnection;

use crate::{
  config::Config,
  handlers::Handler,
  model::{specs::Http, status::*, Check, Duration, Event},
  stash::Stash,
};

pub struct HttpHandler<'h> {
  pub check: &'h Check,
}

#[async_trait]
impl<'h> Handler for HttpHandler<'h> {
  type Spec = Http;

  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>, site: &str, stash: Stash) -> Result<Event> {
    let spec = Http::for_check(conn, self.check).await.context("no spec found for check {}")?;

    self.run(&spec, site, stash).await
  }

  async fn run(&self, spec: &Http, site: &str, _stash: Stash) -> Result<Event> {
    let timeout = spec.timeout.unwrap_or_else(|| Duration::from(5));
    let mut request = ureq::AgentBuilder::new().timeout(*timeout).build().get(&spec.url).set("user-agent", "defcon");

    for (header, value) in spec.headers.iter() {
      request = request.set(header, value);
    }

    let start = Instant::now();
    let response = request.call();
    let duration = start.elapsed();

    let response = match response {
      Ok(response) => Ok(response),
      Err(ureq::Error::Status(_, response)) => Ok(response),
      Err(err) => Err(err),
    };

    let event = match response {
      Ok(response) => {
        let code = response.status();
        let body = response.into_string().unwrap_or_default();

        let code_ok = code == spec.code.unwrap_or(code);
        let content_ok = match spec.content {
          Some(ref content) => body.contains(content),
          None => true,
        };

        let json_ok = {
          match spec.json_query {
            #[allow(unused_variables)]
            Some(ref query) => {
              #[cfg(not(feature = "jq"))]
              {
                log::warn!("http handler `json_query` is used but Defcon was compiled without `jq` feature");
                true
              }

              #[cfg(feature = "jq")]
              jq_rs::run(query, &body).map_or_else(|_| false, |result| result.trim() == "true")
            }

            None => true,
          }
        };

        let digest_ok = match spec.digest {
          Some(ref digest) => {
            let mut hasher = Sha512::new();
            hasher.update(body);
            let result = hasher.finalize();

            digest == &format!("{result:x}")
          }

          None => true,
        };

        let duration_ok = match spec.duration {
          Some(ref maximum) => maximum.0 > duration,
          None => true,
        };

        let (status, message) = match (code_ok, content_ok, digest_ok, json_ok, duration_ok) {
          (false, _, _, _, _) => (CRITICAL, format!("status code was {code}")),
          (_, false, _, _, _) => (CRITICAL, "content mismatch".to_string()),
          (_, _, false, _, _) => (CRITICAL, "digest mismatch".to_string()),
          (_, _, _, false, _) => (CRITICAL, "JSON query failed".to_string()),
          (_, _, _, _, false) => (CRITICAL, "request took too long".to_string()),
          (true, true, true, true, true) => (OK, String::new()),
        };

        Event {
          check_id: self.check.id,
          site: site.to_string(),
          status,
          message,
          ..Default::default()
        }
      }

      Err(err) => Event {
        check_id: self.check.id,
        site: site.to_string(),
        status: CRITICAL,
        message: err.to_string(),
        ..Default::default()
      },
    };

    Ok(event)
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use super::{Handler, HttpHandler};
  use crate::{
    config::CONTROLLER_ID,
    model::{
      specs::{Http, HttpHeaders},
      status::*,
      Check, Duration,
    },
    stash::Stash,
  };

  #[tokio::test]
  async fn handler_http_headers() {
    let mut headers = HashMap::default();
    headers.insert("lorem".to_string(), "ipsum".to_string());

    let handler = HttpHandler { check: &Check::default() };
    let spec = Http {
      id: 0,
      check_id: 0,
      url: "https://httpbin.org/headers".to_string(),
      headers: HttpHeaders(headers),
      timeout: None,
      code: None,
      content: Some(r#""Lorem": "ipsum""#.to_string()),
      digest: None,
      json_query: None,
      duration: None,
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, OK);
    assert_eq!(result.message, String::new());
  }

  #[tokio::test]
  async fn handler_http_ok() {
    let handler = HttpHandler { check: &Check::default() };
    let spec = Http {
      id: 0,
      check_id: 0,
      url: "https://httpbin.org/user-agent".to_string(),
      headers: Default::default(),
      timeout: None,
      code: Some(200),
      content: Some(r#""user-agent": "defcon""#.to_string()),
      digest: Some("2d3cb778b29b905457d6b87b3a4258202bfdbe883251523f7e479e5505b7df6bedbc25f5061e5a677e9e92bf3560a993d5cd88ba5918cc1b5bed1db23b060c84".to_string()),
      json_query: None,
      duration: None,
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, OK);
    assert_eq!(result.message, String::new());
  }

  #[tokio::test]
  async fn handler_http_invalid_status_below_400() {
    let handler = HttpHandler { check: &Check::default() };
    let spec = Http {
      id: 0,
      check_id: 0,
      url: "https://httpbin.org/status/300".to_string(),
      headers: Default::default(),
      timeout: None,
      code: Some(301),
      content: None,
      digest: None,
      json_query: None,
      duration: None,
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
    assert_eq!(result.message, "status code was 300".to_string());
  }

  #[tokio::test]
  async fn handler_http_invalid_status_above_400() {
    let handler = HttpHandler { check: &Check::default() };
    let spec = Http {
      id: 0,
      check_id: 0,
      url: "https://httpbin.org/status/400".to_string(),
      headers: Default::default(),
      timeout: None,
      code: Some(201),
      content: None,
      digest: None,
      json_query: None,
      duration: None,
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
    assert_eq!(result.message, "status code was 400".to_string());
  }

  #[tokio::test]
  async fn handler_http_invalid_content() {
    let handler = HttpHandler { check: &Check::default() };
    let spec = Http {
      id: 0,
      check_id: 0,
      url: "http://httpbin.org/anything/helloworld".to_string(),
      headers: Default::default(),
      code: None,
      timeout: None,
      content: Some("INVALIDCONTENT".to_string()),
      digest: None,
      json_query: None,
      duration: None,
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
    assert_eq!(result.message, "content mismatch".to_string());
  }

  #[tokio::test]
  async fn handler_http_invalid_digest() {
    let handler = HttpHandler { check: &Check::default() };
    let spec = Http {
      id: 0,
      check_id: 0,
      url: "https://httpbin.org/status/200".to_string(),
      headers: Default::default(),
      code: None,
      timeout: None,
      content: None,
      digest: Some("INVALIDDIGEST".to_string()),
      json_query: None,
      duration: None,
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
    assert_eq!(result.message, "digest mismatch".to_string());
  }

  #[cfg(feature = "jq")]
  #[tokio::test]
  async fn handler_http_valid_json_query() {
    let handler = HttpHandler { check: &Check::default() };
    let spec = Http {
      id: 0,
      check_id: 0,
      url: "https://login.microsoftonline.com/common/v2.0/.well-known/openid-configuration".to_string(),
      headers: Default::default(),
      timeout: Some(Duration::from(1)),
      code: Some(200),
      content: None,
      digest: None,
      json_query: Some(r#".claims_supported | contains(["email"])"#.to_string()),
      duration: None,
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[cfg(feature = "jq")]
  #[tokio::test]
  async fn handler_http_invalid_json_query() {
    let handler = HttpHandler { check: &Check::default() };
    let spec = Http {
      id: 0,
      check_id: 0,
      url: "https://login.microsoftonline.com/common/v2.0/.well-known/openid-configuration".to_string(),
      headers: Default::default(),
      timeout: Some(Duration::from(1)),
      code: Some(200),
      content: None,
      digest: None,
      json_query: Some(r#".issuer == "github.com""#.to_string()),
      duration: None,
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
  }

  #[tokio::test]
  async fn handler_http_duration_ok() {
    let handler = HttpHandler { check: &Check::default() };
    let spec = Http {
      id: 0,
      check_id: 0,
      url: "https://httpbin.org/delay/1".to_string(),
      headers: Default::default(),
      code: None,
      timeout: None,
      content: None,
      digest: None,
      json_query: None,
      duration: Some(Duration::from(5)),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[tokio::test]
  async fn handler_http_duration_too_long() {
    let handler = HttpHandler { check: &Check::default() };
    let spec = Http {
      id: 0,
      check_id: 0,
      url: "https://httpbin.org/delay/3".to_string(),
      headers: Default::default(),
      code: None,
      timeout: None,
      content: None,
      digest: None,
      json_query: None,
      duration: Some(Duration::from(1)),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
    assert_eq!(result.message, "request took too long".to_string());
  }

  #[tokio::test]
  async fn handler_http_timeout() {
    let handler = HttpHandler { check: &Check::default() };
    let spec = Http {
      id: 0,
      check_id: 0,
      url: "http://192.0.2.1".to_string(),
      headers: Default::default(),
      timeout: Some(Duration::from(1)),
      code: Some(200),
      content: None,
      digest: None,
      json_query: None,
      duration: None,
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
    assert_eq!(result.message, "http://192.0.2.1/: Connection Failed: Connect error: connection timed out".to_string());
  }
}
