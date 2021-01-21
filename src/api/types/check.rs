use anyhow::{Context, Result};
use futures::{stream, StreamExt};
use serde::Deserialize;
use sqlx::{MySql, Pool};

use crate::{
  api::types::{self as api, ApiMapper},
  model as db,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Check {
  #[serde(flatten)]
  pub check: db::Check,
  pub spec: api::Spec,
  pub alerter: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CheckPatch {
  pub alerter: Option<String>,
  pub name: Option<String>,
  pub enabled: Option<bool>,
  pub interval: Option<i32>,
  pub passing_threshold: Option<i32>,
  pub failing_threshold: Option<i32>,
  pub silent: Option<bool>,
  pub spec: Option<api::Spec>,
}

#[async_trait]
impl ApiMapper for db::Check {
  type Output = api::Check;

  async fn map(self, pool: &Pool<MySql>) -> Result<Self::Output> {
    let mut conn = pool.acquire().await.context("could not retrieve database connection")?;
    let spec = self.spec(&mut *conn).await?;
    let alerter = self.alerter(&mut *conn).await;

    let check = api::Check {
      check: self,
      spec,
      alerter: alerter.map(|alerter| alerter.uuid),
    };

    Ok(check)
  }
}

#[async_trait]
impl ApiMapper for Vec<db::Check> {
  type Output = Vec<api::Check>;

  async fn map(self, pool: &Pool<MySql>) -> Result<Self::Output> {
    let checks: Vec<api::Check> = stream::iter(self)
      .then(async move |check| {
        if let Ok(mut conn) = pool.acquire().await.context("could not retrieve database connection") {
          match check.spec(&mut *conn).await {
            Ok(spec) => {
              let alerter = check.alerter(&mut *conn).await;

              let check = api::Check {
                check,
                spec,
                alerter: alerter.map(|alerter| alerter.uuid),
              };

              Some(check)
            }

            Err(_) => None,
          }
        } else {
          None
        }
      })
      .filter_map(async move |check| check)
      .collect()
      .await;

    Ok(checks)
  }
}
