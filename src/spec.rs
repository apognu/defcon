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
