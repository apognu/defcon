use anyhow::Result;
use sqlx::{FromRow, MySqlConnection};

use crate::model::{
  specs::{http::HttpHeaders, SpecMeta},
  Check, Duration,
};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Http {
  #[serde(skip_serializing, skip_deserializing)]
  pub id: u64,
  #[serde(skip_serializing, skip_deserializing)]
  pub check_id: u64,
  pub url: String,
  #[serde(default)]
  pub headers: HttpHeaders,
  pub timeout: Option<Duration>,
  pub code: Option<u16>,
  pub content: Option<String>,
  pub digest: Option<String>,
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
        SELECT id, check_id, url, timeout, headers, code, content, digest
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
        INSERT INTO http_specs ( check_id, url, headers, timeout, code, content, digest )
        VALUES ( ?, ?, ?, ?, ?, ?, ? )
      ",
    )
    .bind(check.id)
    .bind(spec.url)
    .bind(spec.headers)
    .bind(spec.timeout)
    .bind(spec.code)
    .bind(spec.content)
    .bind(spec.digest)
    .execute(pool)
    .await?;

    Ok(())
  }

  pub async fn update(conn: &mut MySqlConnection, check: &Check, spec: Http) -> Result<()> {
    sqlx::query(
      "
        UPDATE http_specs
        SET url = ?, headers = ?, timeout = ?, code = ?, content = ?, digest = ?
        WHERE check_id = ?
      ",
    )
    .bind(spec.url)
    .bind(spec.headers)
    .bind(spec.timeout)
    .bind(spec.code)
    .bind(spec.content)
    .bind(spec.digest)
    .bind(check.id)
    .execute(conn)
    .await?;

    Ok(())
  }
}
