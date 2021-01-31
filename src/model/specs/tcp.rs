use anyhow::Result;
use sqlx::{FromRow, MySqlConnection};

use crate::model::{specs::SpecMeta, Check, Duration};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Tcp {
  #[serde(skip_serializing, skip_deserializing)]
  pub id: u64,
  #[serde(skip_serializing, skip_deserializing)]
  pub check_id: u64,
  pub host: String,
  pub port: u16,
  pub timeout: Option<Duration>,
}

impl SpecMeta for Tcp {
  fn name(&self) -> &'static str {
    "TCP connection"
  }

  fn fields(&self) -> Vec<(&'static str, String)> {
    vec![("Host", self.host.clone()), ("Port", self.port.to_string())]
  }
}

impl Tcp {
  pub async fn for_check(conn: &mut MySqlConnection, check: &Check) -> Result<Tcp> {
    let spec = sqlx::query_as::<_, Tcp>(
      "
        SELECT id, check_id, host, port, timeout
        FROM tcp_specs
        WHERE check_id = ?
      ",
    )
    .bind(check.id)
    .fetch_one(&mut *conn)
    .await?;

    Ok(spec)
  }

  pub async fn insert(pool: &mut MySqlConnection, check: &Check, spec: Tcp) -> Result<()> {
    sqlx::query(
      "
        INSERT INTO tcp_specs ( check_id, host, port, timeout )
        VALUES ( ?, ?, ?, ? )
      ",
    )
    .bind(check.id)
    .bind(spec.host)
    .bind(spec.port)
    .bind(spec.timeout)
    .execute(pool)
    .await?;

    Ok(())
  }

  pub async fn update(conn: &mut MySqlConnection, check: &Check, spec: Tcp) -> Result<()> {
    sqlx::query(
      "
        UPDATE tcp_specs
        SET host = ?, port = ?, timeout = ?
        WHERE check_id = ?
      ",
    )
    .bind(spec.host)
    .bind(spec.port)
    .bind(spec.timeout)
    .bind(check.id)
    .execute(conn)
    .await?;

    Ok(())
  }
}
