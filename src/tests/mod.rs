mod alerters;
mod checks;
mod db;
mod outages;

use std::env;

use anyhow::Result;
use rocket::{local::asynchronous::Client, Config};
use sqlx::mysql::MySqlPoolOptions;
use url::Url;
use uuid::Uuid;

use crate::{api, model::migrations, tests::TestConnection};

pub use self::{alerters::*, checks::*, db::*, outages::*};

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
