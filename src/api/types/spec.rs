use anyhow::Result;
use sqlx::MySqlConnection;

use crate::{
  api::types::Spec as api,
  model::{
    specs::{self as db, SpecMeta},
    Check as DbCheck, CheckKind,
  },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Spec {
  #[cfg(feature = "ping")]
  #[serde(rename = "ping")]
  Ping(db::Ping),
  #[serde(rename = "dns")]
  Dns(db::Dns),
  #[serde(rename = "http")]
  Http(db::Http),
  #[serde(rename = "tcp")]
  Tcp(db::Tcp),
  #[serde(rename = "udp")]
  Udp(db::Udp),
  #[serde(rename = "tls")]
  Tls(db::Tls),
  #[serde(rename = "play_store")]
  PlayStore(db::PlayStore),
  #[serde(rename = "app_store")]
  AppStore(db::AppStore),
  #[serde(rename = "domain")]
  Whois(db::Whois),
  #[serde(rename = "deadmanswitch")]
  DeadManSwitch(db::DeadManSwitch),
  #[serde(rename = "unsupported")]
  Unsupported,
}

impl Spec {
  pub fn kind(&'_ self) -> CheckKind {
    use CheckKind::*;

    match self {
      #[cfg(feature = "ping")]
      api::Ping(_) => Ping,
      api::Dns(_) => Dns,
      api::Http(_) => Http,
      api::Tcp(_) => Tcp,
      api::Udp(_) => Udp,
      api::Tls(_) => Tls,
      api::PlayStore(_) => PlayStore,
      api::AppStore(_) => AppStore,
      api::Whois(_) => Whois,
      api::DeadManSwitch(_) => DeadManSwitch,
      api::Unsupported => Unsupported,
    }
  }

  pub fn meta(&'_ self) -> &'_ dyn SpecMeta {
    match self {
      #[cfg(feature = "ping")]
      api::Ping(spec) => spec,
      api::Dns(spec) => spec,
      api::Http(spec) => spec,
      api::Tcp(spec) => spec,
      api::Udp(spec) => spec,
      api::Tls(spec) => spec,
      api::PlayStore(spec) => spec,
      api::AppStore(spec) => spec,
      api::Whois(spec) => spec,
      api::DeadManSwitch(spec) => spec,
      api::Unsupported => &db::Unsupported,
    }
  }

  pub async fn insert(self, pool: &mut MySqlConnection, check: &DbCheck) -> Result<()> {
    match self {
      #[cfg(feature = "ping")]
      api::Ping(spec) => db::Ping::insert(pool, check, spec).await,
      api::Dns(spec) => db::Dns::insert(pool, check, spec).await,
      api::Http(spec) => db::Http::insert(pool, check, spec).await,
      api::Tcp(spec) => db::Tcp::insert(pool, check, spec).await,
      api::Udp(spec) => db::Udp::insert(pool, check, spec).await,
      api::Tls(spec) => db::Tls::insert(pool, check, spec).await,
      api::PlayStore(spec) => db::PlayStore::insert(pool, check, spec).await,
      api::AppStore(spec) => db::AppStore::insert(pool, check, spec).await,
      api::Whois(spec) => db::Whois::insert(pool, check, spec).await,
      api::DeadManSwitch(spec) => db::DeadManSwitch::insert(pool, check, spec).await,
      api::Unsupported => Err(anyhow!("cannot insert check with unsupported spec")),
    }
  }

  pub async fn update(self, conn: &mut MySqlConnection, check: &DbCheck) -> Result<()> {
    match self {
      #[cfg(feature = "ping")]
      api::Ping(spec) => db::Ping::update(conn, check, spec).await,
      api::Dns(spec) => db::Dns::update(conn, check, spec).await,
      api::Http(spec) => db::Http::update(conn, check, spec).await,
      api::Tcp(spec) => db::Tcp::update(conn, check, spec).await,
      api::Udp(spec) => db::Udp::update(conn, check, spec).await,
      api::Tls(spec) => db::Tls::update(conn, check, spec).await,
      api::PlayStore(spec) => db::PlayStore::update(conn, check, spec).await,
      api::AppStore(spec) => db::AppStore::update(conn, check, spec).await,
      api::Whois(spec) => db::Whois::update(conn, check, spec).await,
      api::DeadManSwitch(spec) => db::DeadManSwitch::update(conn, check, spec).await,
      api::Unsupported => Err(anyhow!("cannot update check with unsupported spec")),
    }
  }
}
