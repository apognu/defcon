use anyhow::{Context, Result};
use futures::{stream, StreamExt};
use sqlx::{MySql, Pool};

use crate::{
  api::types::{self as api, ApiMapper, CheckGroup},
  model as db,
};

#[derive(Debug, Clone, Serialize)]
pub struct Outage {
  #[serde(flatten)]
  pub outage: db::Outage,
  pub check: api::Check,
}

#[derive(Debug, Deserialize)]
pub struct OutageComment {
  pub comment: String,
}

#[async_trait]
impl ApiMapper for db::Outage {
  type Output = api::Outage;

  async fn map(self, pool: &Pool<MySql>) -> Result<Self::Output> {
    let mut conn = pool.acquire().await.context("could not retrieve database connection")?;
    let check = db::Check::by_id(&mut *conn, self.check_id).await?;
    let spec = check.spec(&mut *conn).await?;
    let group = check.group(&mut *conn).await;
    let alerter = check.alerter(&mut *conn).await;
    let sites = check.sites(&mut *conn).await?;

    let outage = api::Outage {
      outage: self,
      check: api::Check {
        check,
        spec,
        group: CheckGroup::from(group),
        alerter: alerter.map(|alerter| alerter.uuid),
        sites: Some(sites.into()),
      },
    };

    Ok(outage)
  }
}

#[async_trait]
impl ApiMapper for Vec<db::Outage> {
  type Output = Vec<api::Outage>;

  async fn map(self, pool: &Pool<MySql>) -> Result<Self::Output> {
    let outages = stream::iter(self)
      .then(async move |outage| {
        if let Ok(mut conn) = pool.acquire().await.context("could not retrieve database connection") {
          match db::Check::by_id(&mut *conn, outage.check_id).await {
            Ok(check) => match check.spec(&mut *conn).await {
              Ok(spec) => {
                let group = check.group(&mut *conn).await;
                let alerter = check.alerter(&mut *conn).await;
                let sites = check.sites(&mut *conn).await.unwrap_or_default();

                let outage = api::Outage {
                  outage,
                  check: api::Check {
                    check,
                    spec,
                    group: CheckGroup::from(group),
                    alerter: alerter.map(|alerter| alerter.uuid),
                    sites: Some(sites.into()),
                  },
                };

                Some(outage)
              }

              Err(_) => None,
            },

            Err(_) => None,
          }
        } else {
          None
        }
      })
      .filter_map(async move |outage| outage)
      .collect()
      .await;

    Ok(outages)
  }
}
