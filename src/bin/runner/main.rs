mod config;

use std::{sync::Arc, time::Duration};

use anyhow::Result;
use humantime::format_duration;
use kvlogger::*;
use rand::Rng;

use defcon::{
  api::{
    auth::Claims,
    types::{self as api, Spec},
  },
  handlers::*,
  inhibitor::Inhibitor,
  model::Check,
  stash::Stash,
};

use crate::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
  Config::set_log_level()?;

  let config = Config::parse()?;
  let stash = Stash::new();
  let mut inhibitor = Inhibitor::new();

  let claims = Claims {
    site: config.site.clone(),
    ..Default::default()
  };

  kvlog!(Info, "starting runner process", {
    "site" => config.site,
    "poll_interval" => format_duration(config.poll_interval),
    "spread" => config.handler_spread.map(format_duration).map(|s| s.to_string()).unwrap_or_default()
  });

  loop {
    let token = config.keys.generate(&claims)?.unwrap_or_default();
    let client = reqwest::Client::new();
    let request = client.get(&format!("{}/api/runner/checks", config.base)).header("authorization", format!("Bearer {}", token));

    if let Ok(response) = request.send().await {
      if let Ok(checks) = response.json::<Vec<api::RunnerCheck>>().await {
        log::debug!("got {} stale checks from the controller", checks.len());

        let mut rng = rand::thread_rng();

        for check in checks {
          if inhibitor.inhibited(&config.site, &check.uuid).await {
            continue;
          }

          inhibitor.inhibit(&config.site, &check.uuid).await;

          let spread = config.handler_spread.map(|duration| rng.gen_range(0..duration.as_millis() as u64));

          tokio::spawn({
            let config = config.clone();
            let claims = claims.clone();
            let stash = stash.clone();
            let inhibitor = inhibitor.clone();

            async move {
              if let Some(spread) = spread {
                tokio::time::sleep(Duration::from_millis(spread)).await
              }

              let _ = run_check(config, stash, inhibitor, &claims.clone(), check).await;
            }
          });
        }
      }
    }

    tokio::time::sleep(config.poll_interval).await;
  }
}

async fn run_check(config: Arc<Config<'_>>, stash: Stash, mut inhibitor: Inhibitor, claims: &Claims, check: api::RunnerCheck) -> Result<()> {
  let dummy = Check { id: check.id, ..Default::default() };

  let result = match check.spec {
    Spec::Ping(ref spec) => PingHandler { check: &dummy }.run(spec, &config.site, stash).await,

    Spec::Dns(ref spec) => {
      DnsHandler {
        check: &dummy,
        resolver: config.checks.dns_resolver,
      }
      .run(spec, &config.site, stash)
      .await
    }

    Spec::Http(ref spec) => HttpHandler { check: &dummy }.run(spec, &config.site, stash).await,
    Spec::Tcp(ref spec) => TcpHandler { check: &dummy }.run(spec, &config.site, stash).await,
    Spec::Udp(ref spec) => UdpHandler { check: &dummy }.run(spec, &config.site, stash).await,
    Spec::Tls(ref spec) => TlsHandler { check: &dummy }.run(spec, &config.site, stash).await,
    Spec::PlayStore(ref spec) => PlayStoreHandler { check: &dummy }.run(spec, &config.site, stash).await,
    Spec::AppStore(ref spec) => AppStoreHandler { check: &dummy }.run(spec, &config.site, stash).await,
    Spec::Whois(ref spec) => WhoisHandler { check: &dummy }.run(spec, &config.site, stash).await,
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

      inhibitor.release(&config.site, &check.uuid).await;
    }

    Err(_) => inhibitor.inhibit_for(&config.site, &check.uuid, *check.interval).await,
  }

  Ok(())
}
