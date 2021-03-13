use anyhow::Result;
use sqlx::{FromRow, MySqlConnection};

use crate::model::{specs::SpecMeta, Check};

impl SpecMeta for PlayStore {
  fn name(&self) -> &'static str {
    "Play Store app"
  }

  fn fields(&self) -> Vec<(&'static str, String)> {
    vec![("App ID", self.app_id.clone())]
  }
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct PlayStore {
  #[serde(skip)]
  pub id: u64,
  #[serde(skip)]
  pub check_id: u64,
  pub app_id: String,
}

impl PlayStore {
  pub async fn for_check(conn: &mut MySqlConnection, check: &Check) -> Result<PlayStore> {
    let spec = sqlx::query_as::<_, PlayStore>(
      "
        SELECT id, check_id, app_id
        FROM play_store_specs
        WHERE check_id = ?
      ",
    )
    .bind(check.id)
    .fetch_one(&mut *conn)
    .await?;

    Ok(spec)
  }

  pub async fn insert(pool: &mut MySqlConnection, check: &Check, spec: PlayStore) -> Result<()> {
    sqlx::query(
      "
        INSERT INTO play_store_specs ( check_id, app_id )
        VALUES ( ?, ? )
      ",
    )
    .bind(check.id)
    .bind(spec.app_id)
    .execute(pool)
    .await?;

    Ok(())
  }

  pub async fn update(conn: &mut MySqlConnection, check: &Check, spec: PlayStore) -> Result<()> {
    sqlx::query(
      "
        UPDATE play_store_specs
        SET app_id = ?
        WHERE check_id = ?
      ",
    )
    .bind(spec.app_id)
    .bind(check.id)
    .execute(conn)
    .await?;

    Ok(())
  }
}
