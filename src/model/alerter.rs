use anyhow::Result;
use sqlx::{FromRow, MySqlConnection};
use uuid::Uuid;

use crate::{alerters::*, api::error::Shortable, model::AlerterKind};

#[derive(Debug, Default, FromRow, Serialize, Deserialize)]
pub struct Alerter {
  #[serde(skip_serializing, skip_deserializing)]
  pub id: u64,
  #[serde(skip_deserializing)]
  pub uuid: String,
  pub kind: AlerterKind,
  pub webhook: String,
}

impl Alerter {
  pub async fn all(conn: &mut MySqlConnection) -> Result<Vec<Alerter>> {
    let alerters = sqlx::query_as::<_, Alerter>(
      "
        SELECT id, uuid, kind, webhook
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
        SELECT id, uuid, kind, webhook
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
        SELECT id, uuid, kind, webhook
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
        INSERT INTO alerters ( uuid, kind, webhook )
        VALUES ( ?, ?, ? )
      ",
    )
    .bind(&uuid)
    .bind(self.kind)
    .bind(self.webhook)
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
        SET kind = ?, webhook = ?
        WHERE id = ?
      ",
    )
    .bind(self.kind)
    .bind(self.webhook)
    .bind(self.id)
    .execute(&mut *conn)
    .await
    .short()?;

    let alerter = Alerter::by_uuid(conn, &self.uuid).await?;

    Ok(alerter)
  }

  pub fn webhook(self) -> Box<dyn Webhook + Send + Sync> {
    match self.kind {
      AlerterKind::Webhook => Box::new(WebhookAlerter(self)),
      AlerterKind::Slack => Box::new(SlackAlerter(self)),
      AlerterKind::Noop => Box::new(NoopAlerter),
    }
  }
}
