use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, MySqlConnection};
use uuid::Uuid;

use crate::{api::error::Shortable, model::Outage};

#[derive(Debug, Default, FromRow, Clone, Serialize)]
pub struct Timeline {
  #[serde(skip)]
  pub id: u64,
  pub uuid: String,
  #[serde(skip)]
  pub outage_id: u64,
  #[serde(skip)]
  pub user_id: Option<u64>,
  pub kind: String,
  pub content: String,
  pub published_on: Option<DateTime<Utc>>,
}

impl Timeline {
  pub fn new(outage_id: u64, user_id: Option<u64>, kind: &str, content: &str) -> Timeline {
    Timeline {
      outage_id,
      user_id,
      kind: kind.into(),
      content: content.into(),
      ..Default::default()
    }
  }

  pub async fn for_outage(conn: &mut MySqlConnection, outage: &Outage) -> Result<Vec<Timeline>> {
    let timelines = sqlx::query_as::<_, Timeline>(
      "
        SELECT id, uuid, outage_id, user_id, kind, content, published_on
        FROM timelines
        WHERE outage_id = ?
        ORDER BY published_on DESC
      ",
    )
    .bind(outage.id)
    .fetch_all(&mut *conn)
    .await
    .short()?;

    Ok(timelines)
  }

  pub async fn insert(&self, conn: &mut MySqlConnection) -> Result<()> {
    let uuid = Uuid::new_v4().to_string();

    sqlx::query(
      "
        INSERT INTO timelines (uuid, outage_id, user_id, kind, content, published_on)
        VALUES ( ?, ?, ?, ?, ?, NOW() )
      ",
    )
    .bind(&uuid)
    .bind(self.outage_id)
    .bind(self.user_id)
    .bind(&self.kind)
    .bind(&self.content)
    .execute(&mut *conn)
    .await
    .short()?;

    Ok(())
  }
}
