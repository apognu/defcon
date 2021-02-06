use std::{str::FromStr, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::{
  header::{HeaderMap, HeaderName, HeaderValue},
  Client as HttpClient,
};
use sha2::{Digest, Sha512};
use sqlx::MySqlConnection;

use crate::{
  config::Config,
  handlers::Handler,
  model::{specs::Http, status::*, Check, Duration, Event},
};

pub struct HttpHandler<'h> {
  pub check: &'h Check,
}

#[async_trait]
impl<'h> Handler for HttpHandler<'h> {
  type Spec = Http;

  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>, site: &str) -> Result<Event> {
    let spec = Http::for_check(conn, self.check).await.context("no spec found for check {}")?;

    self.run(&spec, site).await
  }

  async fn run(&self, spec: &Http, site: &str) -> Result<Event> {
    let timeout = spec.timeout.unwrap_or_else(|| Duration::from(5));
    let headers: HeaderMap = spec
      .headers
      .iter()
      .map(|(name, value)| (HeaderName::from_str(name), HeaderValue::from_str(value)))
      .filter_map(|(name, value)| match (name, value) {
        (Ok(name), Ok(value)) => Some((name, value)),
        _ => None,
      })
      .collect();

    let client = HttpClient::builder().timeout(*timeout).build()?;
    let response = client.get(&spec.url).headers(headers).send().await;

    let event = match response {
      Ok(response) => {
        let code = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();

        let code_ok = code == spec.code.unwrap_or(code);
        let content_ok = match spec.content {
          Some(ref content) => body.contains(content),
          None => true,
        };
        let digest_ok = match spec.digest {
          Some(ref digest) => {
            let mut hasher = Sha512::new();
            hasher.update(body);
            let result = hasher.finalize();

            digest == &format!("{:x}", result)
          }

          None => true,
        };

        let (status, message) = match (code_ok, content_ok, digest_ok) {
          (false, _, _) => (CRITICAL, format!("status code was {}", code)),
          (_, false, _) => (CRITICAL, "content mismatch".to_string()),
          (_, _, false) => (CRITICAL, "digest mismatch".to_string()),
          (true, true, true) => (OK, String::new()),
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

  use tokio_test::*;

  use super::{Handler, HttpHandler};
  use crate::{
    config::CONTROLLER_ID,
    model::{
      specs::{Http, HttpHeaders},
      status::*,
      Check, Duration,
    },
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
    };

    let result = handler.run(&spec, CONTROLLER_ID).await;
    assert_ok!(&result);

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
      url: "https://example.com".to_string(),
      headers: Default::default(),
      timeout: None,
      code: Some(200),
      content: Some("Example Domain".to_string()),
      digest: Some("d06b93c883f8126a04589937a884032df031b05518eed9d433efb6447834df2596aebd500d69b8283e5702d988ed49655ae654c1683c7a4ae58bfa6b92f2b73a".to_string()),
    };

    let result = handler.run(&spec, CONTROLLER_ID).await;
    assert_ok!(&result);

    let result = result.unwrap();
    assert_eq!(result.status, OK);
    assert_eq!(result.message, String::new());
  }

  #[tokio::test]
  async fn handler_http_invalid_status() {
    let handler = HttpHandler { check: &Check::default() };
    let spec = Http {
      id: 0,
      check_id: 0,
      url: "https://example.com".to_string(),
      headers: Default::default(),
      timeout: None,
      code: Some(201),
      content: None,
      digest: None,
    };

    let result = handler.run(&spec, CONTROLLER_ID).await;
    assert_ok!(&result);

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
    assert_eq!(result.message, "status code was 200".to_string());
  }

  #[tokio::test]
  async fn handler_http_invalid_content() {
    let handler = HttpHandler { check: &Check::default() };
    let spec = Http {
      id: 0,
      check_id: 0,
      url: "https://example.com".to_string(),
      headers: Default::default(),
      code: None,
      timeout: None,
      content: Some("INVALIDCONTENT".to_string()),
      digest: None,
    };

    let result = handler.run(&spec, CONTROLLER_ID).await;
    assert_ok!(&result);

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
      url: "https://example.com".to_string(),
      headers: Default::default(),
      code: None,
      timeout: None,
      content: None,
      digest: Some("INVALIDDIGEST".to_string()),
    };

    let result = handler.run(&spec, CONTROLLER_ID).await;
    assert_ok!(&result);

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
    assert_eq!(result.message, "digest mismatch".to_string());
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
    };

    let result = handler.run(&spec, CONTROLLER_ID).await;
    assert_ok!(&result);

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
    assert_eq!(result.message, "error sending request for url (http://192.0.2.1/): operation timed out".to_string());
  }
}
