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
    let api = env::var("API_ENABLE").unwrap_or_else(|_| "1".into()) == "1";
    let api_port = env::var("API_PORT").unwrap_or_else(|_| "8000".into()).parse::<u16>().unwrap_or(8000);

    let handler = env::var("HANDLER_ENABLE").unwrap_or_else(|_| "1".into()) == "1";
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
