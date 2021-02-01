use std::str::FromStr;

use anyhow::{Context, Result};
use refinery::Report;

mod embedded {
  use refinery::embed_migrations;

  embed_migrations!("src/model/migrations");
}

pub fn migrate(dsn: &str) -> Result<Report> {
  let mut config = refinery::config::Config::from_str(dsn).context("database configuration not found in DSL environment variable")?;
  let report = embedded::migrations::runner().run(&mut config).context("failed to run database migrations")?;

  if !report.applied_migrations().is_empty() {
    log::info!("applied {} database migrations", report.applied_migrations().len());
  }

  Ok(report)
}
