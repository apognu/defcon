use std::{
  net::{Ipv4Addr, Ipv6Addr},
  str::FromStr,
  sync::Arc,
};

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::MySqlConnection;
use trust_dns_client::{
  client::{Client, SyncClient},
  rr::{
    rdata::{caa::Value as CaaValue, CAA},
    DNSClass, Name, RData,
  },
  udp::UdpClientConnection,
};

use crate::{
  config::Config,
  handlers::Handler,
  model::{specs::Dns, status::*, Check, Event},
};

pub struct DnsHandler<'h> {
  pub check: &'h Check,
}

#[async_trait]
impl<'h> Handler for DnsHandler<'h> {
  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>) -> Result<Event> {
    let spec = Dns::for_check(conn, self.check).await.context("no spec found for check")?;

    self.run(spec).await
  }
}

impl<'h> DnsHandler<'h> {
  async fn run(&self, spec: Dns) -> Result<Event> {
    let conn = UdpClientConnection::new("8.8.8.8:53".parse()?)?;
    let client = SyncClient::new(conn);

    let name = Name::from_str(&spec.domain).context("invalid domain")?;
    let response = client.query(&name, DNSClass::IN, spec.record.clone().into()).context("query failed")?;
    let records = response.answers();

    let found = records.iter().fold(Ok(false), |acc: Result<bool, anyhow::Error>, record| match acc {
      Err(_) => acc,
      Ok(true) => acc,
      Ok(false) => match *record.rdata() {
        RData::NS(ref ns) => Ok(ns == &Name::from_str(&spec.value)?),
        RData::MX(ref mx) => Ok(mx.exchange() == &Name::from_str(&spec.value)?),
        RData::A(ref ip) => Ok(ip == &spec.value.parse::<Ipv4Addr>()?),
        RData::AAAA(ref ip) => Ok(ip == &spec.value.parse::<Ipv6Addr>()?),
        RData::CNAME(ref name) => Ok(name == &Name::from_str(&spec.value)?),
        RData::SRV(ref srv) => Ok(srv.target() == &Name::from_str(&spec.value)?),

        RData::CAA(CAA {
          value: CaaValue::Issuer(Some(ref issuer), _),
          ..
        }) => Ok(issuer == &Name::from_str(&spec.value)?),

        _ => Ok(false),
      },
    })?;

    let (status, message) = match found {
      true => (OK, String::new()),
      false => (CRITICAL, format!("{} record for {} did not match {}", spec.record, spec.domain, spec.value)),
    };

    let event = Event {
      check_id: self.check.id,
      status,
      message,
      ..Default::default()
    };

    Ok(event)
  }
}

#[cfg(test)]
mod tests {
  use tokio_test::*;

  use super::DnsHandler;
  use crate::model::{
    specs::{Dns, DnsRecord},
    status::*,
    Check,
  };

  #[tokio::test]
  async fn handler_dns_ns_ok() {
    let handler = DnsHandler { check: &Check::default() };
    let spec = Dns {
      id: 0,
      check_id: 0,
      record: DnsRecord::NS,
      domain: "example.com".to_string(),
      value: "a.iana-servers.net".to_string(),
    };

    let result = handler.run(spec).await;
    assert_ok!(&result);

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[tokio::test]
  async fn handler_dns_mx_ok() {
    let handler = DnsHandler { check: &Check::default() };
    let spec = Dns {
      id: 0,
      check_id: 0,
      record: DnsRecord::MX,
      domain: "github.com".to_string(),
      value: "aspmx.l.google.com".to_string(),
    };

    let result = handler.run(spec).await;
    assert_ok!(&result);

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[tokio::test]
  async fn handler_dns_a_ok() {
    let handler = DnsHandler { check: &Check::default() };
    let spec = Dns {
      id: 0,
      check_id: 0,
      record: DnsRecord::A,
      domain: "example.com".to_string(),
      value: "93.184.216.34".to_string(),
    };

    let result = handler.run(spec).await;
    assert_ok!(&result);

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[tokio::test]
  async fn handler_dns_aaaa_ok() {
    let handler = DnsHandler { check: &Check::default() };
    let spec = Dns {
      id: 0,
      check_id: 0,
      record: DnsRecord::AAAA,
      domain: "example.com".to_string(),
      value: "2606:2800:220:1:248:1893:25c8:1946".to_string(),
    };

    let result = handler.run(spec).await;
    assert_ok!(&result);

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[tokio::test]
  async fn handler_dns_cname_ok() {
    let handler = DnsHandler { check: &Check::default() };
    let spec = Dns {
      id: 0,
      check_id: 0,
      record: DnsRecord::CNAME,
      domain: "www.github.com".to_string(),
      value: "github.com".to_string(),
    };

    let result = handler.run(spec).await;
    assert_ok!(&result);

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[tokio::test]
  async fn handler_dns_caa_ok() {
    let handler = DnsHandler { check: &Check::default() };
    let spec = Dns {
      id: 0,
      check_id: 0,
      record: DnsRecord::CAA,
      domain: "google.com".to_string(),
      value: "pki.goog".to_string(),
    };

    let result = handler.run(spec).await;
    assert_ok!(&result);

    let result = result.unwrap();
    assert_eq!(result.status, OK);
  }

  #[tokio::test]
  async fn handler_dns_ns_critical() {
    let handler = DnsHandler { check: &Check::default() };
    let spec = Dns {
      id: 0,
      check_id: 0,
      record: DnsRecord::A,
      domain: "example.com".to_string(),
      value: "1.2.3.4".to_string(),
    };

    let result = handler.run(spec).await;
    assert_ok!(&result);

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
  }

  #[tokio::test]
  async fn handler_dns_ns_invalid() {
    let handler = DnsHandler { check: &Check::default() };
    let spec = Dns {
      id: 0,
      check_id: 0,
      record: DnsRecord::A,
      domain: "example.com".to_string(),
      value: "example.com".to_string(),
    };

    let result = handler.run(spec).await;
    assert_err!(&result);
  }
}
