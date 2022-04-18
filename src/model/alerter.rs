use anyhow::Result;
use sqlx::{FromRow, MySqlConnection};
use uuid::Uuid;

use crate::{alerters::*, api::error::Shortable, model::AlerterKind};

#[derive(Debug, Default, FromRow, Clone, Serialize, Deserialize)]
pub struct Alerter {
  #[serde(skip)]
  pub id: u64,
  #[serde(skip_deserializing)]
  pub uuid: String,
  pub name: String,
  pub kind: AlerterKind,
  pub webhook: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub username: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub password: Option<String>,
}

impl Alerter {
  pub async fn all(conn: &mut MySqlConnection) -> Result<Vec<Alerter>> {
    let alerters = sqlx::query_as::<_, Alerter>(
      "
        SELECT id, uuid, name, kind, webhook, username, password
        FROM alerters
      ",
    )
    .fetch_all(&mut *conn)
    .await
    .short()?;

    Ok(alerters)
  }

  pub async fn by_id(conn: &mut MySqlConnection, id: u64) -> Result<Alerter> {
    let alerter = sqlx::query_as::<_, Alerter>(
      "
        SELECT id, uuid, name, kind, webhook, username, password
        FROM alerters
        WHERE id = ?
      ",
    )
    .bind(id)
    .fetch_one(&mut *conn)
    .await
    .short()?;

    Ok(alerter)
  }

  pub async fn by_uuid(conn: &mut MySqlConnection, uuid: &str) -> Result<Alerter> {
    let alerter = sqlx::query_as::<_, Alerter>(
      "
        SELECT id, uuid, name, kind, webhook, username, password
        FROM alerters
        WHERE uuid = ?
      ",
    )
    .bind(uuid)
    .fetch_one(&mut *conn)
    .await
    .short()?;

    Ok(alerter)
  }

  pub async fn insert(self, conn: &mut MySqlConnection) -> Result<Alerter> {
    let uuid = Uuid::new_v4().to_string();

    sqlx::query(
      "
        INSERT INTO alerters ( uuid, name, kind, webhook, username, password )
        VALUES ( ?, ?, ?, ?, ?, ? )
      ",
    )
    .bind(&uuid)
    .bind(&self.name)
    .bind(self.kind)
    .bind(self.webhook)
    .bind(self.username)
    .bind(self.password)
    .execute(&mut *conn)
    .await
    .short()?;

    let alerter = Alerter::by_uuid(&mut *conn, &uuid).await?;

    Ok(alerter)
  }

  pub async fn update(self, conn: &mut MySqlConnection) -> Result<Alerter> {
    sqlx::query(
      "
        UPDATE alerters
        SET name = ?, kind = ?, webhook = ?, username = ?, password = ?
        WHERE id = ?
      ",
    )
    .bind(self.name)
    .bind(self.kind)
    .bind(self.webhook)
    .bind(self.username)
    .bind(self.password)
    .bind(self.id)
    .execute(&mut *conn)
    .await
    .short()?;

    let alerter = Alerter::by_uuid(conn, &self.uuid).await?;

    Ok(alerter)
  }

  pub async fn delete(conn: &mut MySqlConnection, uuid: &str) -> Result<()> {
    sqlx::query(
      "
      DELETE FROM alerters
      WHERE uuid = ?
    ",
    )
    .bind(uuid)
    .execute(conn)
    .await
    .short()?;

    Ok(())
  }

  pub fn webhook(self) -> Box<dyn Webhook + Send + Sync> {
    match self.kind {
      AlerterKind::Webhook => Box::new(WebhookAlerter(self)),
      AlerterKind::Slack => Box::new(SlackAlerter(self)),
      AlerterKind::Noop => Box::new(NoopAlerter),
    }
  }
}
