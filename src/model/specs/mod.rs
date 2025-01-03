mod app_store;
mod deadmanswitch;
mod dns;
mod http;
#[cfg(feature = "ping")]
mod ping;
mod play_store;
#[cfg(feature = "python")]
mod python;
mod tcp;
mod tls;
mod udp;
mod unsupported;
mod whois;

#[cfg(feature = "ping")]
pub use self::ping::Ping;
#[cfg(feature = "python")]
pub use self::python::Python;
pub use self::{
  app_store::AppStore,
  deadmanswitch::DeadManSwitch,
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
