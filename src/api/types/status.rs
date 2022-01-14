#[derive(Debug, Default, Serialize)]
pub struct Status {
  pub ok: bool,
  pub checks: i64,
  pub outages: StatusOutages,
}

#[derive(Debug, Default, Serialize)]
pub struct StatusOutages {
  pub site: i64,
  pub global: i64,
}