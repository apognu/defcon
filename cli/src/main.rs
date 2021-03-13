mod checks;
mod cli;

use anyhow::Result;

use clap::Clap;
use colored::*;

use crate::cli::{ChecksCommand, Command, Opts};

pub struct App {
  pub endpoint: String,
}

impl App {
  pub fn error(&self, line: &str) {
    eprintln!("{} {}", "[x]".red(), line);
  }
}

#[tokio::main]
async fn main() -> Result<()> {
  let opts = Opts::parse();
  let app = App { endpoint: opts.endpoint };

  let result: Result<()> = match opts.command {
    Command::Checks(checks) => match checks.command {
      ChecksCommand::List(opts) => checks::list(&app, opts.all).await,
      ChecksCommand::Show(opts) => checks::show(&app, &opts.uuid).await,
    },
  };

  if let Err(err) = &result {
    app.error(&err.to_string());
  }

  Ok(())
}
