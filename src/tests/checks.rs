use anyhow::Result;

use crate::tests::db::TestConnection;

impl TestConnection {
  pub async fn create_check(&self, id: Option<u64>, uuid: Option<String>, name: &str, enabled: Option<bool>, sites: Option<&[&str]>) -> Result<()> {
    let id = match id {
      Some(id) => id,
      None => 1,
    };

    let uuid = match uuid {
      Some(uuid) => uuid,
      None => "dd9a531a-1b0b-4a12-bc09-e5637f916261".to_string(),
    };

    let enabled = match enabled {
      Some(enabled) => enabled,
      None => true,
    };

    let sites = match sites {
      None => &["@controller"],
      Some(sites) => sites,
    };

    sqlx::query(
      r#"
        INSERT INTO checks (id, uuid, enabled, name, kind, `interval`, site_threshold, passing_threshold, failing_threshold)
        VALUES ( ?, ?, ?, ?, "tcp", 10, ?, 2, 2 )
      "#,
    )
    .bind(id)
    .bind(&uuid)
    .bind(enabled)
    .bind(name)
    .bind(sites.len() as u64)
    .execute(&**self)
    .await?;

    for site in sites {
      sqlx::query(r#"INSERT INTO check_sites (check_id, slug) VALUES ( ?, ? )"#).bind(id).bind(site).execute(&**self).await?;
    }

    sqlx::query(r#"INSERT INTO tcp_specs (check_id, host, port, timeout) VALUES ( ?, "0.0.0.0", 80, 10 )"#)
      .bind(id)
      .execute(&**self)
      .await?;

    Ok(())
  }
}
