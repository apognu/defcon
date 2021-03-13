use clap::{crate_authors, crate_version, Clap};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
pub struct Opts {
  #[clap(long, about = "URL to your Defcon controller")]
  pub endpoint: String,
  #[clap(subcommand)]
  pub command: Command,
}

#[derive(Clap)]
pub enum Command {
  #[clap(about = "Manage checks")]
  Checks(Checks),
}

#[derive(Clap)]
pub struct Checks {
  #[clap(subcommand)]
  pub command: ChecksCommand,
}

#[derive(Clap)]
pub enum ChecksCommand {
  #[clap(about = "List checks")]
  List(ChecksListOpts),
  Show(ChecksShowOpts),
}

#[derive(Clap)]
pub struct ChecksListOpts {
  #[clap(long)]
  pub all: bool,
}

#[derive(Clap)]
pub struct ChecksShowOpts {
  pub uuid: String,
}
