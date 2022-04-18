use std::{env, sync::Arc, time::Duration};

use anyhow::{anyhow, Context, Result};
use kvlogger::KvLoggerBuilder;
use once_cell::sync::Lazy;
use regex::Regex;

use defcon::{api::auth::Keys, config::ChecksConfig, ext::EnvExt};

static PRIVATE_KEY: Lazy<Vec<u8>> = Lazy::new(|| {
  env::var("PRIVATE_KEY")
    .map(std::fs::read_to_string)
    .context("PRIVATE_KEY must be provided")
    .unwrap()
    .context("could not read private key")
    .unwrap()
    .as_bytes()
    .to_vec()
});

#[derive(Clone)]
pub struct Config {
  pub base: String,
  pub site: String,
  pub keys: Keys,

  pub poll_interval: Duration,
  pub handler_spread: Option<Duration>,

  pub checks: ChecksConfig,
}

impl Config {
  pub fn set_log_level() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
      env::set_var("RUST_LOG", "defcon=info");
    }

    Ok(KvLoggerBuilder::default().init()?)
  }

  pub fn parse() -> Result<Arc<Config>> {
    let base = env::var("CONTROLLER_URL").context("CONTROLLER_URL should be provided")?;
    let site = env::var("SITE").context("SITE should be provided")?;
    let keys = Keys::new_private(&PRIVATE_KEY).context("PRIVATE_KEY should be provided and be en ECDSA key in PEM format")?;
    let poll_interval = env::var("POLL_INTERVAL").or_duration_min("1s", Duration::from_secs(1)).context("POLL_INTERVAL is not a duration")?;

    let handler_spread = match env::var("HANDLER_SPREAD").or_duration_min("0s", Duration::from_secs(0)).context("HANDLER_SPREAD is not a duration")? {
      duration if duration == Duration::from_nanos(0) => None,
      duration => Some(duration),
    };

    let rgx = Regex::new(r"^[a-z0-9-]+$").unwrap();
    if !rgx.is_match(&site) {
      return Err(anyhow!("SITE should only contain lowercase alphanumeric characters and dashes"));
    }

    let checks = ChecksConfig::new()?;

    let config = Config {
      base,
      site,
      keys,
      poll_interval,
      handler_spread,
      checks,
    };

    Ok(Arc::new(config))
  }
}
