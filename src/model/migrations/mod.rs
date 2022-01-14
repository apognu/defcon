use std::{env, str::FromStr};

use anyhow::{Context, Result};
use refinery::Report;

mod embedded {
  use refinery::embed_migrations;

  embed_migrations!("src/model/migrations");
}

pub fn migrate(dsn: &str, force: bool) -> Result<(bool, Report)> {
  let apply = force || env::args().nth(1).as_deref() == Some("migrate");
  let mut config = refinery::config::Config::from_str(dsn).context("database configuration not found in DSL environment variable")?;

  let pending = embedded::migrations::runner()
    .get_applied_migrations(&mut config)
    .map(|migrations| migrations.iter().any(|migration| migration.applied_on().is_none()))
    .unwrap_or(true);

  if pending && !apply {
    return Err(anyhow!("unapplied migrations pending, aborting"));
  }

  let report = embedded::migrations::runner().run(&mut config).context("failed to run database migrations")?;

  if !report.applied_migrations().is_empty() {
    log::info!("applied {} database migrations", report.applied_migrations().len());
  }

  Ok((apply, report))
}
