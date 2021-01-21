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
  model::{specs::Http, status::*, Check, Event},
};

pub struct HttpHandler<'h> {
  pub check: &'h Check,
}

#[async_trait]
impl<'h> Handler for HttpHandler<'h> {
  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>) -> Result<Event> {
    let spec = Http::for_check(conn, self.check).await.context("no spec found for check {}")?;

    let headers: HeaderMap = spec
      .headers
      .iter()
      .map(|(name, value)| (HeaderName::from_str(name), HeaderValue::from_str(value)))
      .filter_map(|(name, value)| match (name, value) {
        (Ok(name), Ok(value)) => Some((name, value)),
        _ => None,
      })
      .collect();

    let client = HttpClient::new();
    let response = client.get(&spec.url).headers(headers).send().await;

    let event = match response {
      Ok(response) => {
        let code = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();

        let code_ok = code == spec.code.unwrap_or(200);
        let content_ok = match spec.content {
          Some(content) => body.contains(&content),
          None => true,
        };
        let digest_ok = match spec.digest {
          Some(digest) => {
            let mut hasher = Sha512::new();
            hasher.update(body);
            let result = hasher.finalize();

            digest == format!("{:x}", result)
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
          status,
          message,
          ..Default::default()
        }
      }

      Err(err) => Event {
        check_id: self.check.id,
        status: 1,
        message: err.to_string(),
        ..Default::default()
      },
    };

    Ok(event)
  }
}
