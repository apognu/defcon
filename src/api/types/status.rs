use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};

#[derive(Debug, Default, Clone, Serialize)]
pub struct Status {
  pub ok: bool,
  pub checks: i64,
  pub outages: StatusOutages,
  pub status_page: bool,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct StatusOutages {
  pub site: i64,
  pub global: i64,
}

#[derive(Serialize)]
pub struct StatusPage {
  pub ok: bool,
  pub outages: i64,
  pub checks: Vec<StatusPageCheck>,
}

#[derive(Serialize)]
pub struct StatusPageCheck {
  pub name: String,
  pub kind: String,
  pub ok: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub down_since: Option<DateTime<Utc>>,
  pub stats: HashMap<NaiveDate, u64>,
}
