mod alerters;
mod checks;
mod db;
mod groups;
mod outages;

use std::{
  env,
  net::{IpAddr, Ipv4Addr, SocketAddr},
  sync::Arc,
  time::Duration,
};

use anyhow::Result;
use rocket::{local::asynchronous::Client, Config as RocketConfig};
use sqlx::mysql::MySqlPoolOptions;
use url::Url;
use uuid::Uuid;

use crate::{
  api::{self, auth::Keys},
  config::*,
  model::migrations,
  tests::TestConnection,
};

pub use self::{alerters::*, checks::*, db::*, outages::*};

pub fn config() -> Arc<Config> {
  let config = Config {
    api: ApiConfig {
      enable: true,
      listen: "127.0.0.1:1234".parse::<SocketAddr>().unwrap(),
    },
    #[cfg(feature = "web")]
    web: WebConfig {
      enable: false,
      listen: "127.0.0.1:4321".parse::<SocketAddr>().unwrap(),
    },
    handler: HandlerConfig {
      enable: true,
      interval: Duration::from_secs(0),
      spread: Some(Duration::from_secs(0)),
    },
    cleaner: CleanerConfig {
      enable: true,
      interval: Duration::from_secs(0),
      threshold: Duration::from_secs(0),
    },
    dms: DmsConfig {
      enable: true,
      listen: "127.0.0.1:1234".parse::<SocketAddr>().unwrap(),
    },
    checks: ChecksConfig {
      dns_resolver: IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
    },
    alerters: AlertersConfig { default: None, fallback: None },
    key: None,
  };

  Arc::new(config)
}

pub async fn api_client() -> Result<(TestConnection, Client)> {
  let database = format!("defcon_test_{}", Uuid::new_v4().to_simple());
  let mut dsn = Url::parse(&env::var("DSN")?)?;

  let pool = MySqlPoolOptions::new().connect(&dsn.to_string()).await?;
  sqlx::query(&format!("CREATE DATABASE {}", &database)).execute(&pool).await?;

  dsn.set_path(&format!("/{}", database));
  let pool = MySqlPoolOptions::new().connect(&dsn.to_string()).await?;

  migrations::migrate(&dsn.to_string(), true)?;

  let keys = Keys::new_public(
    "-----BEGIN PUBLIC KEY-----MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEMUdYFmfbi57NV7pTIht38+w8yPly7rmrD1MPXenlCOu8Mu5623/ztsGeTV9uatuMQeMS+a7NEFzPGjMIKiR3AA==-----END PUBLIC KEY-----".as_bytes(),
  )?;

  let server = api::server(RocketConfig::default(), config(), pool.clone(), Some(keys));

  Ok((TestConnection(pool, database), Client::untracked(server).await?))
}

pub async fn db_client() -> Result<TestConnection> {
  let database = format!("defcon_test_{}", Uuid::new_v4().to_simple());
  let mut dsn = Url::parse(&env::var("DSN")?)?;

  let pool = MySqlPoolOptions::new().connect(&dsn.to_string()).await?;
  sqlx::query(&format!("CREATE DATABASE {}", &database)).execute(&pool).await?;

  dsn.set_path(&format!("/{}", database));
  let pool = MySqlPoolOptions::new().connect(&dsn.to_string()).await?;

  migrations::migrate(&dsn.to_string(), true)?;

  Ok(TestConnection(pool, database))
}
