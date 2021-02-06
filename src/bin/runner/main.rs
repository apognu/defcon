mod config;

use std::sync::Arc;

use anyhow::Result;
use humantime::format_duration;
use kvlogger::*;

use defcon::{
  api::{
    auth::Claims,
    types::{self as api, Spec},
  },
  handlers::*,
  inhibitor::Inhibitor,
  model::Check,
};

use crate::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
  Config::set_log_level()?;

  let config = Config::parse()?;
  let inhibitor = Inhibitor::new();

  let claims = Claims {
    site: config.site.clone(),
    ..Default::default()
  };

  kvlog!(Info, "starting runner process", {
    "site" => config.site,
    "poll_interval" => format_duration(config.poll_interval)
  });

  loop {
    let token = config.keys.generate(&claims)?.unwrap_or_default();
    let client = reqwest::Client::new();
    let request = client.get(&format!("{}/api/runner/checks", config.base)).header("authorization", format!("Bearer {}", token));

    if let Ok(response) = request.send().await {
      if let Ok(checks) = response.json::<Vec<api::RunnerCheck>>().await {
        for stale in checks {
          if inhibitor.inhibited(&config.base, &stale.uuid) {
            continue;
          }

          tokio::spawn({
            let config = config.clone();
            let claims = claims.clone();
            let inhibitor = inhibitor.clone();

            async move {
              let _ = run_check(config, inhibitor, &claims.clone(), stale).await;
            }
          });
        }
      }
    }

    tokio::time::delay_for(config.poll_interval).await;
  }
}

async fn run_check(config: Arc<Config<'_>>, mut inhibitor: Inhibitor, claims: &Claims, check: api::RunnerCheck) -> Result<()> {
  inhibitor.inhibit(&config.site, &check.uuid);

  let dummy = Check { id: check.id, ..Default::default() };

  let result = match check.spec {
    Spec::Ping(ref spec) => PingHandler { check: &dummy }.run(spec, &config.site).await,
    Spec::Dns(ref spec) => DnsHandler { check: &dummy }.run(spec, &config.site).await,
    Spec::Http(ref spec) => HttpHandler { check: &dummy }.run(spec, &config.site).await,
    Spec::Tcp(ref spec) => TcpHandler { check: &dummy }.run(spec, &config.site).await,
    Spec::Udp(ref spec) => UdpHandler { check: &dummy }.run(spec, &config.site).await,
    Spec::Tls(ref spec) => TlsHandler { check: &dummy }.run(spec, &config.site).await,
    Spec::PlayStore(ref spec) => PlayStoreHandler { check: &dummy }.run(spec, &config.site).await,
    Spec::AppStore(ref spec) => AppStoreHandler { check: &dummy }.run(spec, &config.site).await,
    Spec::Whois(ref spec) => WhoisHandler { check: &dummy }.run(spec, &config.site).await,
  };

  match result {
    Ok(event) => {
      if event.status == 0 {
        kvlog!(Debug, "check passed", {
          "site" => config.site,
          "kind" => check.spec.kind(),
          "check" => check.uuid,
          "name" => check.name,
          "message" => event.message
        });
      } else {
        kvlog!(Debug, "check failed", {
          "site" => config.site,
          "kind" => check.spec.kind(),
          "check" => check.spec.kind(),
          "name" => check.name,
          "message" => event.message
        });
      }

      let report = api::ReportEvent {
        check: check.uuid.clone(),
        status: event.status,
        message: event.message,
      };

      let token = config.keys.generate(claims)?.unwrap_or_default();
      let client = reqwest::Client::new();
      let request = client
        .post(&format!("{}/api/runner/report", config.base))
        .header("authorization", format!("Bearer {}", token))
        .json(&report);
      let _ = request.send().await;

      inhibitor.release(&config.site, &check.uuid);
    }

    Err(_) => inhibitor.inhibit_for(&config.site, &check.uuid, *check.interval),
  }

  Ok(())
}
