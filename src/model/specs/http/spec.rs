use std::collections::HashMap;

use anyhow::Result;
use sqlx::{FromRow, MySqlConnection};

use crate::model::{
  specs::{http::HttpHeaders, SpecMeta},
  Check, Duration,
};

#[derive(Debug, FromRow, Clone, Serialize, Deserialize)]
pub struct Http {
  #[serde(skip)]
  pub id: u64,
  #[serde(skip)]
  pub check_id: u64,
  pub url: String,
  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub headers: HttpHeaders,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub timeout: Option<Duration>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub code: Option<u16>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub content: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub digest: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub json_query: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub duration: Option<Duration>,
}

impl SpecMeta for Http {
  fn name(&self) -> &'static str {
    "HTTP request"
  }

  fn fields(&self) -> Vec<(&'static str, String)> {
    vec![("URL", self.url.clone())]
  }
}

impl Http {
  pub async fn for_check(conn: &mut MySqlConnection, check: &Check) -> Result<Http> {
    let spec = sqlx::query_as::<_, Http>(
      "
        SELECT id, check_id, url, timeout, headers, code, content, digest, json_query, duration
        FROM http_specs
        WHERE check_id = ?
      ",
    )
    .bind(check.id)
    .fetch_one(&mut *conn)
    .await?;

    Ok(spec)
  }

  pub async fn insert(pool: &mut MySqlConnection, check: &Check, spec: Http) -> Result<()> {
    sqlx::query(
      "
        INSERT INTO http_specs ( check_id, url, headers, timeout, code, content, digest, json_query, duration )
        VALUES ( ?, ?, ?, ?, ?, ?, ?, ?, ? )
      ",
    )
    .bind(check.id)
    .bind(spec.url)
    .bind(spec.headers)
    .bind(spec.timeout)
    .bind(spec.code)
    .bind(spec.content)
    .bind(spec.digest)
    .bind(spec.json_query)
    .bind(spec.duration)
    .execute(pool)
    .await?;

    Ok(())
  }

  pub async fn update(conn: &mut MySqlConnection, check: &Check, spec: Http) -> Result<()> {
    sqlx::query(
      "
        UPDATE http_specs
        SET url = ?, headers = ?, timeout = ?, code = ?, content = ?, digest = ?, json_query = ?, duration = ?
        WHERE check_id = ?
      ",
    )
    .bind(spec.url)
    .bind(spec.headers)
    .bind(spec.timeout)
    .bind(spec.code)
    .bind(spec.content)
    .bind(spec.digest)
    .bind(spec.json_query)
    .bind(spec.duration)
    .bind(check.id)
    .execute(conn)
    .await?;

    Ok(())
  }
}
