use std::collections::HashMap;

use anyhow::Context;
use chrono::{NaiveDate, Utc};
use rocket::{serde::json::Json, State};
use sqlx::{MySql, Pool};

use crate::{
  api::{
    error::Shortable,
    types::{self as api, ApiMapper},
    ApiResponse,
  },
  model::{Check, Outage, SiteOutage},
};

#[get("/api/status")]
pub async fn status(pool: &State<Pool<MySql>>) -> ApiResponse<Json<api::Status>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let checks = Check::count(&mut *conn).await.short()?;
  let global_outages = Outage::count(&mut *conn).await.short()?;
  let site_outages = SiteOutage::count(&mut *conn).await.short()?;

  let status = api::Status {
    ok: site_outages == 0,
    checks,
    outages: api::StatusOutages {
      site: site_outages,
      global: global_outages,
    },
  };

  Ok(Json(status))
}

#[get("/api/statistics?<check>&<from>&<to>")]
pub async fn statistics(pool: &State<Pool<MySql>>, check: Option<String>, from: api::Date, to: api::Date) -> ApiResponse<Json<HashMap<NaiveDate, Vec<api::Outage>>>> {
  let mut conn = pool.acquire().await.context("could not retrieve database connection").short()?;

  let check = match check {
    Some(uuid) => Some(Check::by_uuid(&mut conn, &uuid).await.context("could not retrieve check").short()?),
    None => None,
  };

  let outages = Outage::between(&mut conn, check, from.and_hms(0, 0, 0), to.and_hms(23, 59, 59), None, None)
    .await
    .context("could not retrieve outages")
    .short()?
    .map(&*pool)
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
