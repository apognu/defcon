mod alerters;
mod checks;
mod db;
mod groups;
mod outages;

use std::env;

use anyhow::Result;
use rocket::{local::asynchronous::Client, Config};
use sqlx::mysql::MySqlPoolOptions;
use url::Url;
use uuid::Uuid;

use crate::{
  api::{self, auth::Keys},
  model::migrations,
  tests::TestConnection,
};

pub use self::{alerters::*, checks::*, db::*, outages::*};

pub async fn api_client() -> Result<(TestConnection, Client)> {
  let database = format!("defcon_test_{}", Uuid::new_v4().to_simple());
  let mut dsn = Url::parse(&env::var("DSN")?)?;

  let pool = MySqlPoolOptions::new().connect(&dsn.to_string()).await?;
  sqlx::query(&format!("CREATE DATABASE {}", &database)).execute(&pool).await?;

  dsn.set_path(&format!("/{}", database));
  let pool = MySqlPoolOptions::new().connect(&dsn.to_string()).await?;

  migrations::migrate(&dsn.to_string())?;

  let keys = Keys::new_public(
    "-----BEGIN PUBLIC KEY-----MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEMUdYFmfbi57NV7pTIht38+w8yPly7rmrD1MPXenlCOu8Mu5623/ztsGeTV9uatuMQeMS+a7NEFzPGjMIKiR3AA==-----END PUBLIC KEY-----".as_bytes(),
  )?;

  let server = api::server(Config::default(), pool.clone(), Some(keys));

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
