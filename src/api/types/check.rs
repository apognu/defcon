use anyhow::{Context, Result};
use futures::{stream, StreamExt};
use serde::Deserialize;
use sqlx::{MySql, Pool};

use crate::{
  api::types::{self as api, ApiMapper},
  model as db,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerCheck {
  pub id: u64,
  pub uuid: String,
  pub name: String,
  pub interval: db::Duration,
  pub spec: api::Spec,
}

impl From<Check> for RunnerCheck {
  fn from(check: Check) -> RunnerCheck {
    RunnerCheck {
      id: check.check.id,
      uuid: check.check.uuid,
      name: check.check.name,
      interval: check.check.interval,
      spec: check.spec,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Check {
  #[serde(flatten)]
  pub check: db::Check,
  #[serde(skip_serializing_if = "Option::is_none", skip_deserializing)]
  pub status: Option<bool>,
  pub spec: api::Spec,

  #[serde(skip_serializing_if = "Option::is_none", skip_deserializing)]
  pub group: Option<CheckGroup>,
  #[serde(rename = "group", skip_serializing)]
  pub group_in: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none", skip_deserializing)]
  pub alerter: Option<CheckAlerter>,
  #[serde(rename = "alerter", skip_serializing)]
  pub alerter_in: Option<String>,

  pub sites: Option<api::Sites>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckGroup {
  pub uuid: String,
  pub name: String,
}

impl CheckGroup {
  pub fn from(group: Option<db::Group>) -> Option<CheckGroup> {
    group.map(|group| CheckGroup { uuid: group.uuid, name: group.name })
  }
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckAlerter {
  pub uuid: String,
  pub kind: String,
}

impl CheckAlerter {
  pub fn from(alerter: Option<db::Alerter>) -> Option<CheckAlerter> {
    alerter.map(|alerter| CheckAlerter {
      uuid: alerter.uuid,
      kind: alerter.kind.to_string(),
    })
  }
}

#[derive(Debug, Deserialize)]
pub struct CheckPatch {
  pub group: Option<String>,
  pub alerter: Option<String>,
  pub sites: Option<api::Sites>,
  pub name: Option<String>,
  pub enabled: Option<bool>,
  pub interval: Option<db::Duration>,
  pub site_threshold: Option<u8>,
  pub passing_threshold: Option<u8>,
  pub failing_threshold: Option<u8>,
  pub silent: Option<bool>,
  pub spec: Option<api::Spec>,
}

#[async_trait]
impl ApiMapper for db::Check {
  type Output = api::Check;

  async fn map(self, pool: &Pool<MySql>) -> Result<Self::Output> {
    let mut conn = pool.acquire().await.context("could not retrieve database connection")?;
    let spec = self.spec(&mut *conn).await?;
    let group = self.group(&mut *conn).await;
    let alerter = self.alerter(&mut *conn).await;
    let sites = self.sites(&mut *conn).await?;
    let status = Some(self.ok(&mut conn).await);

    let check = api::Check {
      check: self,
      status,
      spec,
      group: CheckGroup::from(group),
      group_in: None,
      alerter: CheckAlerter::from(alerter),
      alerter_in: None,
      sites: Some(sites.into()),
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
              let group = check.group(&mut *conn).await;
              let alerter = check.alerter(&mut *conn).await;
              let sites = check.sites(&mut *conn).await.unwrap_or_default();
              let status = Some(check.ok(&mut conn).await);

              let check = api::Check {
                check,
                status,
                spec,
                group: CheckGroup::from(group),
                group_in: None,
                alerter: CheckAlerter::from(alerter),
                alerter_in: None,
                sites: Some(sites.into()),
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
