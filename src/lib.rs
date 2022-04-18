#![feature(async_closure, try_find)]
#![deny(clippy::all)]
#![allow(clippy::unit_arg)]

#[macro_use]
extern crate serde;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate anyhow;
#[cfg(feature = "web")]
#[macro_use]
extern crate rust_embed;

pub mod alerters;
pub mod api;
pub mod config;
pub mod ext;
pub mod handlers;
pub mod inhibitor;
pub mod model;
pub mod stash;

#[cfg(feature = "web")]
pub mod web;

#[cfg(test)]
mod tests;
