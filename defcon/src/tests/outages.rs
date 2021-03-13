use anyhow::Result;

use crate::{config::CONTROLLER_ID, tests::TestConnection};

impl TestConnection {
  pub async fn create_unresolved_site_outage(&self, id: Option<u64>, uuid: Option<String>) -> Result<()> {
    let id = match id {
      Some(id) => id,
      None => 1,
    };

    let uuid = match uuid {
      Some(uuid) => uuid,
      None => "dd9a531a-1b0b-4a12-bc09-e5637f916261".to_string(),
    };

    sqlx::query(
      r#"
        INSERT INTO site_outages (id, check_id, uuid, site, passing_strikes, failing_strikes, started_on, ended_on)
        VALUES ( ?, 1, ?, ?, 0, 2, "2021-01-02T00:00:00", NULL )
      "#,
    )
    .bind(id)
    .bind(&uuid)
    .bind(CONTROLLER_ID)
    .execute(&**self)
    .await?;

    sqlx::query(
      r#"
        INSERT INTO events (id, check_id, outage_id, site, status, message, created_at)
        VALUES ( ?, 1, ?, ?, 1, "failure", NOW() )
      "#,
    )
    .bind(id)
    .bind(id)
    .bind(CONTROLLER_ID)
    .execute(&**self)
    .await?;

    Ok(())
  }

  pub async fn create_resolved_site_outage(&self, id: Option<u64>, uuid: Option<String>) -> Result<()> {
    let id = match id {
      Some(id) => id,
      None => 1,
    };

    let uuid = match uuid {
      Some(uuid) => uuid,
      None => "dd9a531a-1b0b-4a12-bc09-e5637f916261".to_string(),
    };

    sqlx::query(
      r#"
        INSERT INTO site_outages (id, check_id, uuid, site, passing_strikes, failing_strikes, started_on, ended_on)
        VALUES ( ?, 1, ?, ?, 0, 2, "2021-01-15T00:00:00", "2021-01-16T23:59:59" )
      "#,
    )
    .bind(id)
    .bind(&uuid)
    .bind(CONTROLLER_ID)
    .execute(&**self)
    .await?;

    sqlx::query(
      r#"
        INSERT INTO events (id, check_id, outage_id, site, status, message, created_at)
        VALUES ( ?, 1, ?, ?, 1, "failure", NOW() )
      "#,
    )
    .bind(id)
    .bind(id)
    .bind(CONTROLLER_ID)
    .execute(&**self)
    .await?;

    Ok(())
  }

  pub async fn create_resolved_outage(&self, id: Option<u64>, uuid: Option<String>) -> Result<()> {
    let id = match id {
      Some(id) => id,
      None => 1,
    };

    let uuid = match uuid {
      Some(uuid) => uuid,
      None => "dd9a531a-1b0b-4a12-bc09-e5637f916261".to_string(),
    };

    sqlx::query(
      r#"
        INSERT INTO outages (id, check_id, uuid, started_on, ended_on, comment)
        VALUES ( ?, 1, ?, NOW(), NOW(), NULL )
      "#,
    )
    .bind(id)
    .bind(&uuid)
    .execute(&**self)
    .await?;

    Ok(())
  }
}
