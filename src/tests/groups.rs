use anyhow::Result;

use crate::tests::db::TestConnection;

impl TestConnection {
  pub async fn create_group(&self, id: Option<u64>, uuid: Option<String>, name: &str) -> Result<()> {
    let id = match id {
      Some(id) => id,
      None => 1,
    };

    let uuid = match uuid {
      Some(uuid) => uuid,
      None => "dd9a531a-1b0b-4a12-bc09-e5637f916261".to_string(),
    };

    sqlx::query(
      "
        INSERT INTO groups (id, uuid, name)
        VALUES ( ?, ?, ? )
      ",
    )
    .bind(id)
    .bind(&uuid)
    .bind(name)
    .execute(&**self)
    .await?;

    Ok(())
  }
}
