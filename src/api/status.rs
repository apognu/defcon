use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use axum::{
  extract::{Query, State},
  Json,
};
use chrono::{NaiveDate, Utc};
use sqlx::{MySql, Pool};

use crate::{
  api::{
    auth::Auth,
    error::Shortable,
    types::{self as api, ApiMapper},
    ApiResponse,
  },
  handlers::Config,
  model::{Check, Outage, SiteOutage},
};

#[allow(unused_variables)]
pub async fn status(_: Auth, config: State<Arc<Config>>, pool: State<Pool<MySql>>) -> ApiResponse<Json<api::Status>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let checks = Check::count(&mut conn).await.short()?;
  let global_outages = Outage::count(&mut conn).await.short()?;
  let site_outages = SiteOutage::count(&mut conn).await.short()?;

  let status = api::Status {
    ok: site_outages == 0,
    checks,
    outages: api::StatusOutages {
      site: site_outages,
      global: global_outages,
    },
    #[cfg(feature = "web")]
    status_page: config.web.enable_status_page,
    #[cfg(not(feature = "web"))]
    status_page: false,
  };

  Ok(Json(status))
}

#[derive(Deserialize)]
pub struct StatisticsQuery {
  check: Option<String>,
  from: api::Date,
  to: api::Date,
}

pub async fn statistics(_: Auth, ref pool: State<Pool<MySql>>, Query(StatisticsQuery { check, from, to }): Query<StatisticsQuery>) -> ApiResponse<Json<HashMap<NaiveDate, Vec<api::Outage>>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let check = match check {
    Some(uuid) => Some(Check::by_uuid(&mut conn, &uuid).await.context("could not retrieve check").short()?),
    None => None,
  };

  let outages = Outage::between(&mut conn, check.as_ref(), from.and_hms(0, 0, 0), to.and_hms(23, 59, 59), None, None)
    .await
    .context("could not retrieve outages")
    .short()?
    .map(pool)
    .await
    .short()?;

  let now = Utc::now().date().naive_local();
  let interval = to.signed_duration_since(*from).num_days() as usize + 1;

  let mut stats: HashMap<NaiveDate, Vec<api::Outage>> = HashMap::new();

  for outage in outages {
    let start = outage.outage.started_on.unwrap().date().naive_local();

    from.iter_days().take(interval).for_each(|day| {
      if day <= now && start <= day && (outage.outage.ended_on.is_none() || outage.outage.ended_on.unwrap().date().naive_local() >= day) {
        stats.entry(day).or_default().push(outage.clone());
      }
    });
  }

  Ok(Json(stats))
}

#[cfg(feature = "web")]
pub async fn status_page(ref pool: State<Pool<MySql>>) -> ApiResponse<Json<api::StatusPage>> {
  use std::ops::Sub;

  use chrono::Duration;
  use futures::{stream, StreamExt};

  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let outages = Outage::count(&mut conn).await.short()?;
  let checks = Check::list(&mut conn, false, true, None, None, None).await.short()?;

  let checks = stream::iter(checks)
    .then(async move |check| {
      if let Ok(mut conn) = pool.acquire().await.context("could not retrieve database connection") {
        let outage = Outage::for_check_current(&mut conn, &check).await;
        let to = Utc::now().date().naive_utc();
        let from = Utc::now().sub(Duration::days(30)).date().naive_utc();

        let outages = Outage::between(&mut conn, Some(&check), from.and_hms(0, 0, 0), to.and_hms(23, 59, 59), None, None)
          .await
          .context("could not retrieve outages")
          .ok()?
          .map(pool)
          .await
          .ok()?;

        let now = Utc::now().date().naive_local();
        let interval = to.signed_duration_since(from).num_days() as usize + 1;

        let mut stats: HashMap<NaiveDate, u64> = HashMap::new();

        for outage in outages {
          let start = outage.outage.started_on.unwrap().date().naive_local();

          from.iter_days().take(interval).for_each(|day| {
            if day <= now && start <= day && (outage.outage.ended_on.is_none() || outage.outage.ended_on.unwrap().date().naive_local() >= day) {
              *stats.entry(day).or_default() += 1;
            }
          });
        }

        Some(api::StatusPageCheck {
          name: check.name.clone(),
          kind: check.kind.to_string(),
          ok: outage.is_err(),
          down_since: outage.map(|outage| outage.started_on).ok().flatten(),
          stats,
        })
      } else {
        None
      }
    })
    .filter_map(async move |check| check)
    .collect::<Vec<_>>()
    .await;

  let status = api::StatusPage { ok: outages == 0, outages, checks };

  Ok(Json(status))
}
