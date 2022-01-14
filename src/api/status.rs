use anyhow::Context;
use rocket::{serde::json::Json, State};
use sqlx::{MySql, Pool};

use crate::{
  api::{error::Shortable, types as api, ApiResponse},
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
