use std::{
  net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
  str::FromStr,
  sync::Arc,
};

use anyhow::{Context, Result};
use async_trait::async_trait;
use hickory_client::{
  client::{AsyncClient, ClientHandle},
  rr::{
    rdata::{caa::Value as CaaValue, CAA},
    DNSClass, Name, RData,
  },
  udp::UdpClientStream,
};
use sqlx::MySqlConnection;
use tokio::net::UdpSocket;

use crate::{
  config::Config,
  handlers::Handler,
  model::{specs::Dns, status::*, Check, Event},
  stash::Stash,
};

pub struct DnsHandler<'h> {
  pub check: &'h Check,
  pub resolver: IpAddr,
}

#[async_trait]
impl Handler for DnsHandler<'_> {
  type Spec = Dns;

  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>, site: &str, stash: Stash) -> Result<Event> {
    let spec = Dns::for_check(conn, self.check).await.context("no spec found for check")?;

    self.run(&spec, site, stash).await
  }

  async fn run(&self, spec: &Dns, site: &str, _stash: Stash) -> Result<Event> {
    let resolver = SocketAddr::new(self.resolver, 53);
    let conn = UdpClientStream::<UdpSocket>::new(resolver);
    let (mut client, task) = AsyncClient::connect(conn).await?;

    tokio::spawn(task);

    let name = Name::from_str(&spec.domain).context("invalid domain")?;
    let response = client.query(name, DNSClass::IN, spec.record.clone().into()).await.context("query failed")?;
    let records = response.answers();

    let found = records.iter().try_fold(false, |acc, record| -> Result<_> {
      match acc {
        true => Ok(acc),
        false => match record.data() {
          Some(RData::NS(ref ns)) => Ok(ns.0 == Name::from_str(&spec.value)?),
          Some(RData::MX(ref mx)) => Ok(mx.exchange() == &Name::from_str(&spec.value)?),
          Some(RData::A(ref ip)) => Ok(ip.0 == spec.value.parse::<Ipv4Addr>()?),
          Some(RData::AAAA(ref ip)) => Ok(ip.0 == spec.value.parse::<Ipv6Addr>()?),
          Some(RData::CNAME(ref name)) => Ok(name.0 == Name::from_str(&spec.value)?),
          Some(RData::SRV(ref srv)) => Ok(srv.target() == &Name::from_str(&spec.value)?),

          Some(RData::CAA(CAA {
            value: CaaValue::Issuer(Some(ref issuer), _),
            ..
          })) => Ok(issuer == &Name::from_str(&spec.value)?),

          _ => Ok(false),
        },
      }
    })?;

    let (status, message) = match found {
      true => (OK, String::new()),
      false => (CRITICAL, format!("{} record for {} did not match {}", spec.record, spec.domain, spec.value)),
    };

    let event = Event {
      check_id: self.check.id,
      site: site.to_string(),
      status,
      message,
      ..Default::default()
    };

    Ok(event)
  }
}

#[cfg(test)]
mod tests {
  use std::net::{IpAddr, Ipv4Addr};

  use super::{DnsHandler, Handler};
  use crate::{
    config::CONTROLLER_ID,
    model::{
      specs::{Dns, DnsRecord},
      status::*,
      Check,
    },
    stash::Stash,
  };

  #[tokio::test]
  async fn handler_dns_ns_ok() {
    let handler = DnsHandler {
      check: &Check::default(),
      resolver: IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
    };

    let spec = Dns {
      id: 0,
      check_id: 0,
      record: DnsRecord::NS,
      domain: "example.com".to_string(),
      value: "a.iana-servers.net".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[tokio::test]
  async fn handler_dns_mx_ok() {
    let handler = DnsHandler {
      check: &Check::default(),
      resolver: IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
    };

    let spec = Dns {
      id: 0,
      check_id: 0,
      record: DnsRecord::MX,
      domain: "github.com".to_string(),
      value: "aspmx.l.google.com".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[tokio::test]
  async fn handler_dns_a_ok() {
    let handler = DnsHandler {
      check: &Check::default(),
      resolver: IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
    };

    let spec = Dns {
      id: 0,
      check_id: 0,
      record: DnsRecord::A,
      domain: "example.com".to_string(),
      value: "93.184.215.14".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[tokio::test]
  async fn handler_dns_aaaa_ok() {
    let handler = DnsHandler {
      check: &Check::default(),
      resolver: IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
    };

    let spec = Dns {
      id: 0,
      check_id: 0,
      record: DnsRecord::AAAA,
      domain: "example.com".to_string(),
      value: "2606:2800:21f:cb07:6820:80da:af6b:8b2c".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[tokio::test]
  async fn handler_dns_cname_ok() {
    let handler = DnsHandler {
      check: &Check::default(),
      resolver: IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
    };

    let spec = Dns {
      id: 0,
      check_id: 0,
      record: DnsRecord::CNAME,
      domain: "www.github.com".to_string(),
      value: "github.com".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[tokio::test]
  async fn handler_dns_caa_ok() {
    let handler = DnsHandler {
      check: &Check::default(),
      resolver: IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
    };

    let spec = Dns {
      id: 0,
      check_id: 0,
      record: DnsRecord::CAA,
      domain: "google.com".to_string(),
      value: "pki.goog".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[tokio::test]
  async fn handler_dns_ns_critical() {
    let handler = DnsHandler {
      check: &Check::default(),
      resolver: IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
    };
    let spec = Dns {
      id: 0,
      check_id: 0,
      record: DnsRecord::A,
      domain: "example.com".to_string(),
      value: "1.2.3.4".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
  }

  #[tokio::test]
  async fn handler_dns_ns_invalid() {
    let handler = DnsHandler {
      check: &Check::default(),
      resolver: IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
    };

    let spec = Dns {
      id: 0,
      check_id: 0,
      record: DnsRecord::A,
      domain: "example.com".to_string(),
      value: "example.com".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Err(_)));
  }
}
