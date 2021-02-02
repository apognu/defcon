use std::{
  env,
  ops::{Deref, DerefMut},
};

use anyhow::Result;
use rocket::{local::asynchronous::Client, Config};
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use url::Url;
use uuid::Uuid;

use crate::{api, model::migrations};

#[derive(Clone)]
pub struct TestConnection(pub Pool<MySql>, pub String);

impl Deref for TestConnection {
  type Target = Pool<MySql>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for TestConnection {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl TestConnection {
  pub async fn cleanup(self) {
    sqlx::query(&format!("DROP DATABASE {}", self.1)).execute(&self.0).await.unwrap();
  }

  pub async fn create_check(&self, id: Option<u64>, uuid: Option<String>, name: &str, enabled: Option<bool>) -> Result<()> {
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

    sqlx::query(
      r#"
        INSERT INTO checks (id, uuid, enabled, name, kind, `interval`, site_threshold, passing_threshold, failing_threshold)
        VALUES ( ?, ?, ?, ?, "tcp", 10, 2, 2, 2 )
      "#,
    )
    .bind(id)
    .bind(&uuid)
    .bind(enabled)
    .bind(name)
    .execute(&**self)
    .await?;

    sqlx::query(r#"INSERT INTO check_sites (check_id, slug) VALUES ( ?, "@controller" )"#).bind(id).execute(&**self).await?;

    sqlx::query(r#"INSERT INTO tcp_specs (check_id, host, port, timeout) VALUES ( ?, "0.0.0.0", 80, 10 )"#)
      .bind(id)
      .execute(&**self)
      .await?;

    Ok(())
  }

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

  pub async fn create_unresolved_outage(&self, id: Option<u64>, uuid: Option<String>) -> Result<()> {
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
        VALUES ( ?, 1, ?, "@controller", 0, 2, "2021-01-02T00:00:00", NULL )
      "#,
    )
    .bind(id)
    .bind(&uuid)
    .execute(&**self)
    .await?;

    sqlx::query(
      r#"
        INSERT INTO events (id, check_id, outage_id, site, status, message, created_at)
        VALUES ( ?, 1, ?, "@controller", 1, "failure", NOW() )
      "#,
    )
    .bind(id)
    .bind(id)
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
        INSERT INTO site_outages (id, check_id, uuid, site, passing_strikes, failing_strikes, started_on, ended_on)
        VALUES ( ?, 1, ?, "@controller", 0, 2, "2021-01-15T00:00:00", "2021-01-16T23:59:59" )
      "#,
    )
    .bind(id)
    .bind(&uuid)
    .execute(&**self)
    .await?;

    sqlx::query(
      r#"
        INSERT INTO events (id, check_id, outage_id, site, status, message, created_at)
        VALUES ( ?, 1, ?, "@controller", 1, "failure", NOW() )
      "#,
    )
    .bind(id)
    .bind(id)
    .execute(&**self)
    .await?;

    Ok(())
  }
}

pub async fn api_client() -> Result<(TestConnection, Client)> {
  let database = format!("defcon_test_{}", Uuid::new_v4().to_simple());
  let mut dsn = Url::parse(&env::var("DSN")?)?;

  let pool = MySqlPoolOptions::new().connect(&dsn.to_string()).await?;
  sqlx::query(&format!("CREATE DATABASE {}", &database)).execute(&pool).await?;

  dsn.set_path(&format!("/{}", database));
  let pool = MySqlPoolOptions::new().connect(&dsn.to_string()).await?;

  migrations::migrate(&dsn.to_string())?;

  let server = api::server(Config::default(), pool.clone());

  Ok((TestConnection(pool, database), Client::untracked(server).await?))
}

pub async fn db_client() -> Result<TestConnection> {
  let database = format!("defcon_test_{}", Uuid::new_v4().to_simple());
  let mut dsn = Url::parse(&env::var("DSN")?)?;

  let pool = MySqlPoolOptions::new().connect(&dsn.to_string()).await?;
  sqlx::query(&format!("CREATE DATABASE {}", &database)).execute(&pool).await?;

  dsn.set_path(&format!("/{}", database));
  let pool = MySqlPoolOptions::new().connect(&dsn.to_string()).await?;

  migrations::migrate(&dsn.to_string())?;

  Ok(TestConnection(pool, database))
}
