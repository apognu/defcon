mod app_store;
mod dns;
mod http;
mod ping;
mod play_store;
mod tcp;
mod tls;
mod udp;
mod whois;

pub use self::{
  app_store::AppStore,
  dns::{Dns, DnsRecord},
  http::{Http, HttpHeaders},
  ping::Ping,
  play_store::PlayStore,
  tcp::Tcp,
  tls::Tls,
  udp::Udp,
  whois::Whois,
};

pub trait SpecMeta {
  fn name(&self) -> &'static str;
  fn fields(&self) -> Vec<(&'static str, String)>;
}
