use std::{
  env,
  net::{IpAddr, Ipv4Addr, SocketAddr},
  sync::Arc,
  time::Duration,
};

use anyhow::{Context, Result};
use humantime::parse_duration;
use kvlogger::KvLoggerBuilder;

use crate::ext::EnvExt;

#[derive(Debug, Clone)]
pub struct Config {
  pub api: bool,
  pub api_port: u16,

  pub handler: bool,
  pub handler_interval: Duration,
  pub handler_spread: Option<Duration>,

  pub cleaner: bool,
  pub cleaner_interval: Duration,
  pub cleaner_threshold: Duration,

  pub checks: ChecksConfig,

  pub key: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ChecksConfig {
  pub dns_resolver: SocketAddr,
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
    let api_port = env::var("API_PORT").or_string("8000").parse::<u16>().unwrap_or(8000);

    let handler = env::var("HANDLER_ENABLE").or_string("1") == "1";
    let handler_interval = parse_duration(&env::var("HANDLER_INTERVAL").or_string("1s")).context("HANDLER_INTERVAL is not a duration")?;

    let handler_spread = match parse_duration(&env::var("HANDLER_SPREAD").or_string("0s")).context("HANDLER_SPREAD is not a duration")? {
      duration if duration == Duration::from_nanos(0) => None,
      duration => Some(duration),
    };

    let cleaner = env::var("CLEANER_ENABLE").unwrap_or_default() == "1";
    let cleaner_interval = parse_duration(&env::var("CLEANER_INTERVAL").or_string("10m")).context("CLEANER_INTERVAL is not a duration")?;
    let cleaner_threshold = parse_duration(&env::var("CLEANER_THRESHOLD").or_string("1y")).context("CLEANER_THRESHOLD is not a duration")?;

    let dns_resolver = match env::var("DNS_RESOLVER") {
      Ok(resolver) => SocketAddr::new(resolver.parse().context("DNS_RESOLVER is not an IP address")?, 53),
      Err(_) => SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 53),
    };

    let checks = ChecksConfig { dns_resolver };

    let key = env::var("PUBLIC_KEY")
      .map(|key| format!("-----BEGIN PUBLIC KEY-----{}-----END PUBLIC KEY-----", key))
      .context("PUBLIC_KEY must be provided")?
      .as_bytes()
      .to_vec();

    let config = Config {
      api,
      api_port,
      handler,
      handler_interval,
      handler_spread,
      cleaner,
      cleaner_interval,
      cleaner_threshold,
      checks,
      key,
    };

    Ok(Arc::new(config))
  }
}

#[cfg(test)]
mod tests {
  use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
  };

  use anyhow::Result;
  use jsonwebtoken::DecodingKey;
  use serial_test::serial;

  use super::Config;

  fn add_public_key() {
    env::set_var(
      "PUBLIC_KEY",
      "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEMUdYFmfbi57NV7pTIht38+w8yPly7rmrD1MPXenlCOu8Mu5623/ztsGeTV9uatuMQeMS+a7NEFzPGjMIKiR3AA==",
    );
  }

  #[test]
  #[serial]
  fn default_config() -> Result<()> {
    add_public_key();

    let config = Config::parse()?;

    assert_eq!(config.api, true);
    assert_eq!(config.api_port, 8000);
    assert_eq!(config.handler, true);
    assert_eq!(config.handler_interval, Duration::from_secs(1));
    assert_eq!(config.handler_spread, None);
    assert_eq!(config.cleaner, false);
    assert_eq!(config.cleaner_interval, Duration::from_secs(600));
    assert_eq!(config.cleaner_threshold, Duration::from_secs(31557600));

    assert_eq!(config.checks.dns_resolver, SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 53));

    assert_eq!(config.key.len(), 174);
    assert!(matches!(DecodingKey::from_ec_pem(&config.key), Ok(_)));

    Ok(())
  }

  #[test]
  #[serial]
  fn fails_without_public_key() -> Result<()> {
    env::remove_var("PUBLIC_KEY");

    assert!(matches!(Config::parse(), Err(_)));

    Ok(())
  }

  #[test]
  #[serial]
  fn fails_with_invalid_public_key() -> Result<()> {
    env::set_var("PUBLIC_KEY", "invalidkey");

    let config = Config::parse()?;

    assert!(matches!(DecodingKey::from_ec_pem(&config.key), Err(_)));

    Ok(())
  }

  #[test]
  #[serial]
  fn override_config() -> Result<()> {
    add_public_key();

    env::set_var("API_ENABLE", "0");
    env::set_var("API_PORT", "10000");
    env::set_var("HANDLER_ENABLE", "0");
    env::set_var("HANDLER_INTERVAL", "10s");
    env::set_var("HANDLER_SPREAD", "10s");
    env::set_var("CLEANER_ENABLE", "1");
    env::set_var("CLEANER_INTERVAL", "10s");
    env::set_var("CLEANER_THRESHOLD", "10s");

    let config = Config::parse()?;

    assert_eq!(config.api, false);
    assert_eq!(config.api_port, 10000);
    assert_eq!(config.handler, false);
    assert_eq!(config.handler_interval, Duration::from_secs(10));
    assert_eq!(config.handler_spread, Some(Duration::from_secs(10)));
    assert_eq!(config.cleaner, true);
    assert_eq!(config.cleaner_interval, Duration::from_secs(10));
    assert_eq!(config.cleaner_threshold, Duration::from_secs(10));

    env::remove_var("API_ENABLE");
    env::remove_var("API_PORT");
    env::remove_var("HANDLER_ENABLE");
    env::remove_var("HANDLER_INTERVAL");
    env::remove_var("HANDLER_SPREAD");
    env::remove_var("CLEANER_ENABLE");
    env::remove_var("CLEANER_INTERVAL");
    env::remove_var("CLEANER_THRESHOLD");

    Ok(())
  }
}
