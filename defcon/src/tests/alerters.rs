use anyhow::Result;

use crate::tests::TestConnection;

impl TestConnection {
  pub async fn create_alerter(&self) -> Result<()> {
    sqlx::query(
      r#"
        INSERT INTO alerters (uuid, kind, webhook)
        VALUES ( "dd9a531a-1b0b-4a12-bc09-e5637f916261", "webhook", "https://webhooks.example.com/1" )
      "#,
    )
    .execute(&**self)
    .await?;

    Ok(())
  }
}
