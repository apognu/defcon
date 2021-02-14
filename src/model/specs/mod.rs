mod app_store;
mod dns;
mod http;
#[cfg(feature = "ping")]
mod ping;
mod play_store;
mod tcp;
mod tls;
mod udp;
mod unsupported;
mod whois;

#[cfg(feature = "ping")]
pub use self::ping::Ping;
pub use self::{
  app_store::AppStore,
  dns::{Dns, DnsRecord},
  http::{Http, HttpHeaders},
  play_store::PlayStore,
  tcp::Tcp,
  tls::Tls,
  udp::Udp,
  unsupported::Unsupported,
  whois::Whois,
};

pub trait SpecMeta {
  fn name(&self) -> &'static str;
  fn fields(&self) -> Vec<(&'static str, String)>;
}
