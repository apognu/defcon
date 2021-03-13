use anyhow::Result;
use sqlx::{FromRow, MySqlConnection};

use crate::model::{specs::SpecMeta, Check};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct AppStore {
  #[serde(skip)]
  pub id: u64,
  #[serde(skip)]
  pub check_id: u64,
  pub bundle_id: String,
}

impl SpecMeta for AppStore {
  fn name(&self) -> &'static str {
    "App Store app"
  }

  fn fields(&self) -> Vec<(&'static str, String)> {
    vec![("Bundle ID", self.bundle_id.clone())]
  }
}

impl AppStore {
  pub async fn for_check(conn: &mut MySqlConnection, check: &Check) -> Result<AppStore> {
    let spec = sqlx::query_as::<_, AppStore>(
      "
        SELECT id, check_id, bundle_id
        FROM app_store_specs
        WHERE check_id = ?
      ",
    )
    .bind(check.id)
    .fetch_one(conn)
    .await?;

    Ok(spec)
  }

  pub async fn insert(pool: &mut MySqlConnection, check: &Check, spec: AppStore) -> Result<()> {
    sqlx::query(
      "
        INSERT INTO app_store_specs ( check_id, bundle_id )
        VALUES ( ?, ? )
      ",
    )
    .bind(check.id)
    .bind(spec.bundle_id)
    .execute(pool)
    .await?;

    Ok(())
  }

  pub async fn update(conn: &mut MySqlConnection, check: &Check, spec: AppStore) -> Result<()> {
    sqlx::query(
      "
        UPDATE app_store_specs
        SET bundle_id = ?
        WHERE check_id = ?
      ",
    )
    .bind(spec.bundle_id)
    .bind(check.id)
    .execute(conn)
    .await?;

    Ok(())
  }
}
