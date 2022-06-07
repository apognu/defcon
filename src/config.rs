use std::{
  env, fs,
  net::{IpAddr, Ipv4Addr, SocketAddr},
  sync::Arc,
  time::Duration,
};

use anyhow::{Context, Result};
use kvlogger::KvLoggerBuilder;
use once_cell::sync::Lazy;

use crate::ext::EnvExt;

pub const CONTROLLER_ID: &str = "@controller";

pub static PUBLIC_KEY: Lazy<Option<Vec<u8>>> = Lazy::new(|| {
  env::var("PUBLIC_KEY")
    .map(|key| fs::read_to_string(key).expect("could not read public key"))
    .map(|key| key.as_bytes().to_vec())
    .ok()
});

#[derive(Debug, Clone)]
pub struct Config {
  pub api: ApiConfig,
  #[cfg(feature = "web")]
  pub web: WebConfig,
  pub handler: HandlerConfig,
  pub cleaner: CleanerConfig,
  pub dms: DmsConfig,
  pub checks: ChecksConfig,
  pub alerters: AlertersConfig,
  pub key: Option<&'static Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct ApiConfig {
  pub enable: bool,
  pub listen: SocketAddr,
}

impl ApiConfig {
  pub fn new() -> Result<ApiConfig> {
    let enable = env::var("API_ENABLE").or_string("1") == "1";
    let listen = env::var("API_LISTEN").or_string("127.0.0.1:8000").parse::<SocketAddr>().context("could not parse API listen address")?;

    Ok(ApiConfig { enable, listen })
  }
}

#[derive(Debug, Clone)]
pub struct WebConfig {
  pub enable: bool,
  pub listen: SocketAddr,
}

impl WebConfig {
  pub fn new() -> Result<WebConfig> {
    let enable = env::var("WEB_ENABLE").or_string("0") == "1";
    let listen = env::var("WEB_LISTEN").or_string("127.0.0.1:3000").parse::<SocketAddr>().context("could not parse Web listen address")?;

    Ok(WebConfig { enable, listen })
  }
}

#[derive(Debug, Clone)]
pub struct HandlerConfig {
  pub enable: bool,
  pub interval: Duration,
  pub spread: Option<Duration>,
}

impl HandlerConfig {
  pub fn new() -> Result<HandlerConfig> {
    let enable = env::var("HANDLER_ENABLE").or_string("1") == "1";
    let interval = env::var("HANDLER_INTERVAL")
      .or_duration_min("1s", Duration::from_secs(1))
      .context("HANDLER_INTERVAL is not a duration")?;

    let spread = match env::var("HANDLER_SPREAD").or_duration_min("0s", Duration::from_secs(0)).context("HANDLER_SPREAD is not a duration")? {
      duration if duration == Duration::from_nanos(0) => None,
      duration => Some(duration),
    };

    Ok(HandlerConfig { enable, interval, spread })
  }
}

#[derive(Debug, Clone)]
pub struct CleanerConfig {
  pub enable: bool,
  pub interval: Duration,
  pub threshold: Duration,
}

impl CleanerConfig {
  pub fn new() -> Result<CleanerConfig> {
    let enable = env::var("CLEANER_ENABLE").unwrap_or_default() == "1";
    let interval = env::var("CLEANER_INTERVAL")
      .or_duration_min("10m", Duration::from_secs(1))
      .context("CLEANER_INTERVAL is not a duration")?;

    let threshold = env::var("CLEANER_THRESHOLD")
      .or_duration_min("1y", Duration::from_secs(1))
      .context("CLEANER_THRESHOLD is not a duration")?;

    Ok(CleanerConfig { enable, interval, threshold })
  }
}

#[derive(Debug, Clone)]
pub struct DmsConfig {
  pub enable: bool,
  pub listen: SocketAddr,
}

impl DmsConfig {
  pub fn new() -> Result<DmsConfig> {
    let enable = env::var("DMS_ENABLE").or_string("1") == "1";
    let listen = env::var("DMS_LISTEN")
      .or_string("127.0.0.1:8080")
      .parse::<SocketAddr>()
      .context("could not parse Dead Man Switch listen address")?;

    Ok(DmsConfig { enable, listen })
  }
}

#[derive(Debug, Clone)]
pub struct ChecksConfig {
  pub dns_resolver: IpAddr,
  #[cfg(feature = "python")]
  pub scripts_path: String,
}

impl ChecksConfig {
  pub fn new() -> Result<ChecksConfig> {
    let resolver = match env::var("DNS_RESOLVER") {
      Ok(resolver) => SocketAddr::new(resolver.parse().context("DNS_RESOLVER is not an IP address")?, 53),
      Err(_) => SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 53),
    };

    #[cfg(feature = "python")]
    let scripts_path = env::var("SCRIPTS_PATH").or_string("/var/lib/defcon/scripts");

    Ok(ChecksConfig {
      dns_resolver: resolver.ip(),
      #[cfg(feature = "python")]
      scripts_path,
    })
  }
}

#[derive(Debug, Clone)]
pub struct AlertersConfig {
  pub default: Option<String>,
  pub fallback: Option<String>,
}

impl Default for AlertersConfig {
  fn default() -> AlertersConfig {
    AlertersConfig {
      default: env::var("ALERTER_DEFAULT").ok(),
      fallback: env::var("ALERTER_FALLBACK").ok(),
    }
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
    let config = Config {
      api: ApiConfig::new()?,
      #[cfg(feature = "web")]
      web: WebConfig::new()?,
      handler: HandlerConfig::new()?,
      cleaner: CleanerConfig::new()?,
      dms: DmsConfig::new()?,
      checks: ChecksConfig::new()?,
      alerters: AlertersConfig::default(),
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

    assert_eq!(config.api.enable, true);
    assert_eq!(config.api.listen.ip(), Ipv4Addr::new(127, 0, 0, 1));
    assert_eq!(config.api.listen.port(), 8000);
    #[cfg(feature = "web")]
    {
      assert_eq!(config.web.enable, false);
      assert_eq!(config.web.listen.ip(), Ipv4Addr::new(127, 0, 0, 1));
      assert_eq!(config.web.listen.port(), 3000);
    }
    assert_eq!(config.handler.enable, true);
    assert_eq!(config.handler.interval, Duration::from_secs(1));
    assert_eq!(config.handler.spread, None);
    assert_eq!(config.cleaner.enable, false);
    assert_eq!(config.cleaner.interval, Duration::from_secs(600));
    assert_eq!(config.cleaner.threshold, Duration::from_secs(31557600));

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
    #[cfg(feature = "web")]
    {
      env::set_var("WEB_ENABLE", "1");
      env::set_var("WEB_LISTEN", "0.0.0.0:10001");
    }
    env::set_var("HANDLER_ENABLE", "0");
    env::set_var("HANDLER_INTERVAL", "10s");
    env::set_var("HANDLER_SPREAD", "10s");
    env::set_var("CLEANER_ENABLE", "1");
    env::set_var("CLEANER_INTERVAL", "10s");
    env::set_var("CLEANER_THRESHOLD", "10s");

    let config = Config::parse()?;

    assert_eq!(config.api.enable, false);
    assert_eq!(config.api.listen.ip(), Ipv4Addr::new(0, 0, 0, 0));
    assert_eq!(config.api.listen.port(), 10000);
    #[cfg(feature = "web")]
    {
      assert_eq!(config.web.enable, true);
      assert_eq!(config.web.listen.ip(), Ipv4Addr::new(0, 0, 0, 0));
      assert_eq!(config.web.listen.port(), 10001);
    }
    assert_eq!(config.handler.enable, false);
    assert_eq!(config.handler.interval, Duration::from_secs(10));
    assert_eq!(config.handler.spread, Some(Duration::from_secs(10)));
    assert_eq!(config.cleaner.enable, true);
    assert_eq!(config.cleaner.interval, Duration::from_secs(10));
    assert_eq!(config.cleaner.threshold, Duration::from_secs(10));

    env::remove_var("API_ENABLE");
    env::remove_var("API_LISTEN");
    #[cfg(feature = "web")]
    {
      env::remove_var("WEB_ENABLE");
      env::remove_var("WEB_LISTEN");
    }
    env::remove_var("HANDLER_ENABLE");
    env::remove_var("HANDLER_INTERVAL");
    env::remove_var("HANDLER_SPREAD");
    env::remove_var("CLEANER_ENABLE");
    env::remove_var("CLEANER_INTERVAL");
    env::remove_var("CLEANER_THRESHOLD");

    Ok(())
  }
}
