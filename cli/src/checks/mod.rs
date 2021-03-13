mod display;

use anyhow::Result;
use colored::*;
use defcon::{api::types as api, model::Duration};
use humantime::format_duration;
use prettytable::{cell, format::consts::*, row, Cell, Row, Table};
use serde::Deserialize;

use crate::{checks::display::DisplaySpec, App};

#[derive(Debug, Deserialize)]
pub struct Group {
  pub uuid: String,
  pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Check {
  pub uuid: String,
  pub name: String,
  pub group: Option<Group>,
  pub enabled: bool,
  pub interval: Duration,
  pub site_threshold: u8,
  pub passing_threshold: u8,
  pub failing_threshold: u8,
  pub silent: bool,
  pub spec: api::Spec,
}

pub async fn list(app: &App, all: bool) -> Result<()> {
  let url = if all {
    format!("{}/api/checks?all=true", app.endpoint)
  } else {
    format!("{}/api/checks", app.endpoint)
  };

  let checks: Vec<Check> = reqwest::get(&url).await?.json().await?;

  let mut table = Table::new();
  table.set_format(*FORMAT_NO_BORDER_LINE_SEPARATOR);
  table.add_row(row!["", b -> "UUID", b -> "Group", b -> "Name"]);

  for check in &checks {
    let enabled = if check.enabled { Cell::new("⚫").style_spec("Fg") } else { Cell::new("⚫").style_spec("Fr") };
    let group = match &check.group {
      None => String::new(),
      Some(group) => group.name.clone(),
    };

    table.add_row(Row::new(vec![enabled, Cell::new(&check.uuid), Cell::new(&group), Cell::new(&check.name)]));
  }

  table.printstd();

  Ok(())
}

pub async fn show(app: &App, uuid: &str) -> Result<()> {
  let url = format!("{}/api/checks/{}", app.endpoint, uuid);
  let check: Check = reqwest::get(&url).await?.json().await?;

  let enabled = if check.enabled { "yes" } else { "no" };
  let silent = if check.silent { "yes" } else { "no" };

  if let Some(group) = check.group {
    println!("{}", group.name.bold());
  }
  println!("{}", check.name);
  println!("{}", check.uuid.dimmed());
  println!();

  println!("{}", "GENERAL".bold());
  println!("---");

  let mut table = Table::new();
  table.set_format(*FORMAT_NO_BORDER_LINE_SEPARATOR);
  table.add_row(row![b -> "Enabled", enabled]);
  table.add_row(row![b -> "Interval", format_duration(check.interval.0)]);
  table.add_row(row![b -> "Site threshold", check.site_threshold]);
  table.add_row(row![b -> "Passing threshold", check.passing_threshold]);
  table.add_row(row![b -> "Failing threshold", check.failing_threshold]);
  table.add_row(row![b -> "Silent", silent]);
  table.printstd();

  println!();
  println!("{}", "SPEC".bold());
  println!("---");

  check.spec.display();

  Ok(())
}
