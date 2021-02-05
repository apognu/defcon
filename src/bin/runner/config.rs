use std::{env, sync::Arc, time::Duration};

use anyhow::{anyhow, Context, Result};
use humantime::parse_duration;
use kvlogger::KvLoggerBuilder;
use regex::Regex;

use defcon::{api::auth::Keys, ext::EnvExt};

#[derive(Debug, Default, Clone)]
pub struct Config<'k> {
  pub base: String,
  pub site: String,
  pub keys: Keys<'k>,

  pub pull_interval: Duration,
}

impl<'k> Config<'k> {
  pub fn set_log_level() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
      env::set_var("RUST_LOG", "defcon=info");
    }

    Ok(KvLoggerBuilder::default().init()?)
  }

  pub fn parse() -> Result<Arc<Config<'k>>> {
    let base = env::var("CONTROLLER_URL").context("CONTROLLER_URL should be provided")?;
    let site = env::var("SITE").context("SITE should be provided")?;
    let keys = Keys::new_private(&crate::PRIVATE_KEY).context("PRIVATE_KEY should be provided and be en ECDSA key in PEM format")?;

    let pull_interval = parse_duration(&env::var("PULL_INTERVAL").or_string("1s")).context("PULL_INTERVAL is not a duration")?;

    let rgx = Regex::new(r"^[a-z0-9-]+$").unwrap();

    if !rgx.is_match(&site) {
      return Err(anyhow!("SITE should only contain lowercase alphanumeric characters and dashes"));
    }

    let config = Config { base, site, keys, pull_interval };

    Ok(Arc::new(config))
  }
}
