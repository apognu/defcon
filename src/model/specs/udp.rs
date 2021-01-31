use anyhow::Result;
use sqlx::{FromRow, MySqlConnection};

use crate::model::{specs::SpecMeta, Binary, Check, Duration};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Udp {
  #[serde(skip_serializing, skip_deserializing)]
  pub id: u64,
  #[serde(skip_serializing, skip_deserializing)]
  pub check_id: u64,
  pub host: String,
  pub port: u16,
  pub message: Binary,
  pub content: Binary,
  pub timeout: Option<Duration>,
}

impl SpecMeta for Udp {
  fn name(&self) -> &'static str {
    "UDP connection"
  }

  fn fields(&self) -> Vec<(&'static str, String)> {
    vec![("Host", self.host.clone()), ("Port", self.port.to_string())]
  }
}

impl Udp {
  pub async fn for_check(conn: &mut MySqlConnection, check: &Check) -> Result<Udp> {
    let spec = sqlx::query_as::<_, Udp>(
      "
        SELECT id, check_id, host, port, message, content, timeout
        FROM udp_specs
        WHERE check_id = ?
      ",
    )
    .bind(check.id)
    .fetch_one(&mut *conn)
    .await?;

    Ok(spec)
  }

  pub async fn insert(pool: &mut MySqlConnection, check: &Check, spec: Udp) -> Result<()> {
    sqlx::query(
      "
        INSERT INTO udp_specs ( check_id, host, port, message, content, timeout )
        VALUES ( ?, ?, ?, ?, ?, ? )
      ",
    )
    .bind(check.id)
    .bind(spec.host)
    .bind(spec.port)
    .bind(spec.message)
    .bind(spec.content)
    .bind(spec.timeout)
    .execute(pool)
    .await?;

    Ok(())
  }

  pub async fn update(conn: &mut MySqlConnection, check: &Check, spec: Udp) -> Result<()> {
    sqlx::query(
      "
        UPDATE udp_specs
        SET host = ?, port = ?, message = ?, content = ?, timeout = ?
        WHERE check_id = ?
      ",
    )
    .bind(spec.host)
    .bind(spec.port)
    .bind(spec.message)
    .bind(spec.content)
    .bind(spec.timeout)
    .bind(check.id)
    .execute(conn)
    .await?;

    Ok(())
  }
}
