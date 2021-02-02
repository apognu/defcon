use anyhow::Result;
use kvlogger::*;
use std::time::Duration;

use defcon::{
  api::types::{self as api, Spec},
  handlers::*,
  inhibitor::Inhibitor,
  model::Check,
};

const BASE: &str = "http://127.0.0.1:8000/api/checks";
const SITE: &str = "eu-1";

#[tokio::main]
async fn main() -> Result<()> {
  KvLoggerBuilder::default().init()?;

  let inhibitor = Inhibitor::new();

  loop {
    let client = reqwest::Client::new();
    let checks: Vec<api::Check> = client.get(&format!("{}/stale?site={}", BASE, SITE)).send().await?.json().await?;

    for stale in checks {
      if inhibitor.inhibited(SITE, &stale.check.uuid) {
        continue;
      }

      tokio::spawn({
        let inhibitor = inhibitor.clone();

        async move {
          let _ = run_check(inhibitor, stale).await;
        }
      });
    }

    tokio::time::delay_for(Duration::from_secs(1)).await;
  }
}

async fn run_check(mut inhibitor: Inhibitor, check: api::Check) -> Result<()> {
  inhibitor.inhibit(SITE, &check.check.uuid);

  let dummy = Check {
    id: check.check.id,
    ..Default::default()
  };

  let result = match check.spec {
    Spec::Ping(ref spec) => PingHandler { check: &dummy }.run(spec, SITE).await,
    Spec::Dns(ref spec) => DnsHandler { check: &dummy }.run(spec, SITE).await,
    Spec::Http(ref spec) => HttpHandler { check: &dummy }.run(spec, SITE).await,
    Spec::Tcp(ref spec) => TcpHandler { check: &dummy }.run(spec, SITE).await,
    Spec::Udp(ref spec) => UdpHandler { check: &dummy }.run(spec, SITE).await,
    Spec::Tls(ref spec) => TlsHandler { check: &dummy }.run(spec, SITE).await,
    Spec::PlayStore(ref spec) => PlayStoreHandler { check: &dummy }.run(spec, SITE).await,
    Spec::AppStore(ref spec) => AppStoreHandler { check: &dummy }.run(spec, SITE).await,
    Spec::Whois(ref spec) => WhoisHandler { check: &dummy }.run(spec, SITE).await,
  };

  match result {
    Ok(event) => {
      if event.status == 0 {
        kvlog!(Debug, "passed", {
          "kind" => check.spec.kind(),
          "check" => check.check.uuid,
          "name" => check.check.name,
          "message" => event.message
        });
      } else {
        kvlog!(Debug, "failed", {
          "kind" => check.spec.kind(),
          "check" => check.spec.kind(),
          "name" => check.check.name,
          "message" => event.message
        });
      }

      let report = api::ReportEvent {
        check: check.check.uuid.clone(),
        status: event.status,
        message: event.message,
      };

      let client = reqwest::Client::new();
      let _ = client.post(&format!("{}/report?site={}", BASE, SITE)).json(&report).send().await;

      inhibitor.release(SITE, &check.check.uuid);
    }

    Err(_) => inhibitor.inhibit_for(SITE, &check.check.uuid, *check.check.interval),
  }

  Ok(())
}
