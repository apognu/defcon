use anyhow::Result;
use sqlx::{FromRow, MySqlConnection};

use crate::model::{specs::SpecMeta, Check, Duration};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Whois {
  #[serde(skip)]
  pub id: u64,
  #[serde(skip)]
  pub check_id: u64,
  pub domain: String,
  pub window: Duration,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub attribute: Option<String>,
}

impl SpecMeta for Whois {
  fn name(&self) -> &'static str {
    "Domain expiration"
  }

  fn fields(&self) -> Vec<(&'static str, String)> {
    vec![("Domain", self.domain.clone())]
  }
}

impl Whois {
  pub async fn for_check(conn: &mut MySqlConnection, check: &Check) -> Result<Whois> {
    let spec = sqlx::query_as::<_, Whois>(
      "
        SELECT id, check_id, domain, window, attribute
        FROM whois_specs
        WHERE check_id = ?
      ",
    )
    .bind(check.id)
    .fetch_one(&mut *conn)
    .await?;

    Ok(spec)
  }

  pub async fn insert(pool: &mut MySqlConnection, check: &Check, spec: Whois) -> Result<()> {
    sqlx::query(
      "
        INSERT INTO whois_specs ( check_id, domain, window, attribute )
        VALUES ( ?, ?, ?, ? )
      ",
    )
    .bind(check.id)
    .bind(spec.domain)
    .bind(spec.window)
    .bind(spec.attribute)
    .execute(pool)
    .await?;

    Ok(())
  }

  pub async fn update(conn: &mut MySqlConnection, check: &Check, spec: Whois) -> Result<()> {
    sqlx::query(
      "
        UPDATE whois_specs
        SET domain = ?, window = ?, attribute = ?
        WHERE check_id = ?
      ",
    )
    .bind(spec.domain)
    .bind(spec.window)
    .bind(spec.attribute)
    .bind(check.id)
    .execute(conn)
    .await?;

    Ok(())
  }
}
