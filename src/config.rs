use std::{
  env, fs,
  net::{IpAddr, Ipv4Addr, SocketAddr},
  sync::Arc,
  time::Duration,
};

use anyhow::{Context, Result};
use kvlogger::KvLoggerBuilder;
use lazy_static::lazy_static;

use crate::ext::EnvExt;

pub const CONTROLLER_ID: &str = "@controller";

lazy_static! {
  pub static ref PUBLIC_KEY: Option<Vec<u8>> = env::var("PUBLIC_KEY")
    .map(|key| fs::read_to_string(key).expect("could not read public key"))
    .map(|key| key.as_bytes().to_vec())
    .ok();
}

#[derive(Debug, Clone)]
pub struct Config {
  pub api: bool,
  pub api_listen: SocketAddr,

  pub handler: bool,
  pub handler_interval: Duration,
  pub handler_spread: Option<Duration>,

  pub cleaner: bool,
  pub cleaner_interval: Duration,
  pub cleaner_threshold: Duration,

  pub dms: bool,
  pub dms_listen: SocketAddr,

  pub checks: ChecksConfig,

  pub key: Option<&'static Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct ChecksConfig {
  pub dns_resolver: IpAddr,
}

impl ChecksConfig {
  pub fn new() -> Result<ChecksConfig> {
    let resolver = match env::var("DNS_RESOLVER") {
      Ok(resolver) => SocketAddr::new(resolver.parse().context("DNS_RESOLVER is not an IP address")?, 53),
      Err(_) => SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 53),
    };

    Ok(ChecksConfig { dns_resolver: resolver.ip() })
  }
}

impl Config {
  pub fn set_log_level() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
      env::set_var("RUST_LOG", "defcon=info");
    }

    Ok(KvLoggerBuilder::default().init()?)
  }

  pub fn parse() -> Result<Arc<Config>> {
    let api = env::var("API_ENABLE").or_string("1") == "1";
    let api_listen = env::var("API_LISTEN").or_string("127.0.0.1:8000").parse::<SocketAddr>().context("could not parse API listen address")?;

    let handler = env::var("HANDLER_ENABLE").or_string("1") == "1";
    let handler_interval = env::var("HANDLER_INTERVAL")
      .or_duration_min("1s", Duration::from_secs(1))
      .context("HANDLER_INTERVAL is not a duration")?;

    let handler_spread = match env::var("HANDLER_SPREAD").or_duration_min("0s", Duration::from_secs(0)).context("HANDLER_SPREAD is not a duration")? {
      duration if duration == Duration::from_nanos(0) => None,
      duration => Some(duration),
    };

    let cleaner = env::var("CLEANER_ENABLE").unwrap_or_default() == "1";
    let cleaner_interval = env::var("CLEANER_INTERVAL")
      .or_duration_min("10m", Duration::from_secs(1))
      .context("CLEANER_INTERVAL is not a duration")?;

    let cleaner_threshold = env::var("CLEANER_THRESHOLD")
      .or_duration_min("1y", Duration::from_secs(1))
      .context("CLEANER_THRESHOLD is not a duration")?;

    let dms= env::var("DMS_ENABLE").or_string("1") == "1";
    let dms_listen = env::var("DMS_LISTEN").or_string("127.0.0.1:8080").parse::<SocketAddr>().context("could not parse Dead Man Switch listen address")?;

    let checks = ChecksConfig::new()?;

    let config = Config {
      api,
      api_listen,
      handler,
      handler_interval,
      handler_spread,
      cleaner,
      cleaner_interval,
      cleaner_threshold,
      dms,
      dms_listen,
      checks,
      key: PUBLIC_KEY.as_ref(),
    };

    Ok(Arc::new(config))
  }
}

#[cfg(test)]
mod tests {
  use std::{env, fs, net::Ipv4Addr, time::Duration};

  use anyhow::Result;
  use jsonwebtoken::DecodingKey;
  use serial_test::serial;

  use super::Config;

  fn write_keys() -> Result<()> {
    fs::write(
      "/tmp/defcon-test-public-valid.pem",
      "-----BEGIN PUBLIC KEY-----MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEMUdYFmfbi57NV7pTIht38+w8yPly7rmrD1MPXenlCOu8Mu5623/ztsGeTV9uatuMQeMS+a7NEFzPGjMIKiR3AA==-----END PUBLIC KEY-----",
    )?;

    env::set_var("PUBLIC_KEY", "/tmp/defcon-test-public-valid.pem");

    Ok(())
  }

  #[test]
  #[serial]
  fn default_config() -> Result<()> {
    write_keys()?;

    let config = Config::parse()?;

    assert_eq!(config.api, true);
    assert_eq!(config.api_listen.ip(), Ipv4Addr::new(127, 0, 0, 1));
    assert_eq!(config.api_listen.port(), 8000);
    assert_eq!(config.handler, true);
    assert_eq!(config.handler_interval, Duration::from_secs(1));
    assert_eq!(config.handler_spread, None);
    assert_eq!(config.cleaner, false);
    assert_eq!(config.cleaner_interval, Duration::from_secs(600));
    assert_eq!(config.cleaner_threshold, Duration::from_secs(31557600));

    assert_eq!(config.checks.dns_resolver, Ipv4Addr::new(1, 1, 1, 1));

    assert!(matches!(config.key, Some(key) if key.len() == 174));
    assert!(matches!(DecodingKey::from_ec_pem(&(config.key.unwrap())), Ok(_)));

    Ok(())
  }

  #[test]
  #[serial]
  fn override_config() -> Result<()> {
    write_keys()?;

    env::set_var("API_ENABLE", "0");
    env::set_var("API_LISTEN", "0.0.0.0:10000");
    env::set_var("HANDLER_ENABLE", "0");
    env::set_var("HANDLER_INTERVAL", "10s");
    env::set_var("HANDLER_SPREAD", "10s");
    env::set_var("CLEANER_ENABLE", "1");
    env::set_var("CLEANER_INTERVAL", "10s");
    env::set_var("CLEANER_THRESHOLD", "10s");

    let config = Config::parse()?;

    assert_eq!(config.api, false);
    assert_eq!(config.api_listen.ip(), Ipv4Addr::new(0, 0, 0, 0));
    assert_eq!(config.api_listen.port(), 10000);
    assert_eq!(config.handler, false);
    assert_eq!(config.handler_interval, Duration::from_secs(10));
    assert_eq!(config.handler_spread, Some(Duration::from_secs(10)));
    assert_eq!(config.cleaner, true);
    assert_eq!(config.cleaner_interval, Duration::from_secs(10));
    assert_eq!(config.cleaner_threshold, Duration::from_secs(10));

    env::remove_var("API_ENABLE");
    env::remove_var("API_LISTEN");
    env::remove_var("HANDLER_ENABLE");
    env::remove_var("HANDLER_INTERVAL");
    env::remove_var("HANDLER_SPREAD");
    env::remove_var("CLEANER_ENABLE");
    env::remove_var("CLEANER_INTERVAL");
    env::remove_var("CLEANER_THRESHOLD");

    Ok(())
  }
}
