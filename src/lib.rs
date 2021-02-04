#![feature(async_closure, try_find)]
#![deny(clippy::all)]
#![allow(clippy::unit_arg)]

#[macro_use]
extern crate serde;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate anyhow;

pub mod alerters;
pub mod api;
pub mod config;
pub mod ext;
pub mod handlers;
pub mod inhibitor;
pub mod model;

#[cfg(test)]
mod tests;

use anyhow::Error;

pub fn log_error(err: &Error) {
  // let desc = err.to_string();
  // let cause = err.root_cause().to_string();

  log::error!("{:#}", err);
}
