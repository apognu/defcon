use anyhow::Result;
use sqlx::{FromRow, MySqlConnection};

use crate::model::Check;

#[derive(Debug, FromRow)]
pub struct Site {
  pub check_id: u64,
  pub slug: String,
}

impl Site {
  pub async fn insert(conn: &mut MySqlConnection, check: &Check, sites: &[String]) -> Result<()> {
    sqlx::query("DELETE FROM check_sites WHERE check_id = ?").bind(check.id).execute(&mut *conn).await?;

    for slug in sites {
      sqlx::query("INSERT INTO check_sites (check_id, slug) VALUES ( ?, ? )")
        .bind(check.id)
        .bind(slug)
        .execute(&mut *conn)
        .await?;
    }

    Ok(())
  }
}
