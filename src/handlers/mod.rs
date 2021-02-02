mod app_store;
mod dns;
mod http;
mod ping;
mod play_store;
mod tcp;
mod tls;
mod udp;
mod whois;

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use sqlx::MySqlConnection;

pub use crate::{
  config::Config,
  handlers::{app_store::AppStoreHandler, dns::DnsHandler, http::HttpHandler, ping::PingHandler, play_store::PlayStoreHandler, tcp::TcpHandler, tls::TlsHandler, udp::UdpHandler, whois::WhoisHandler},
  model::Event,
};

#[async_trait]
pub trait Handler: Send {
  async fn check(&self, conn: &mut MySqlConnection, config: Arc<Config>, site: &str) -> Result<Event>;
}
