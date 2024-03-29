use anyhow::{Context, Result};
use futures::{stream, StreamExt};
use sqlx::{MySql, Pool};

use crate::{
  api::types::{self as api, ApiMapper, CheckAlerter, CheckGroup},
  model as db,
};

#[derive(Debug, Clone, Serialize)]
pub struct SiteOutage {
  #[serde(flatten)]
  pub outage: db::SiteOutage,
  pub check: api::Check,
}

#[async_trait]
impl ApiMapper for db::SiteOutage {
  type Output = api::SiteOutage;

  async fn map(self, pool: &Pool<MySql>) -> Result<Self::Output> {
    let mut conn = pool.acquire().await.context("could not retrieve database connection")?;
    let check = db::Check::by_id(&mut conn, self.check_id).await?;
    let spec = check.spec(&mut conn).await?;
    let group = check.group(&mut conn).await;
    let alerter = check.alerter(&mut conn).await;
    let sites = check.sites(&mut conn).await?;

    let outage = api::SiteOutage {
      outage: self,
      check: api::Check {
        check,
        status: None,
        spec,
        group: CheckGroup::from(group),
        group_in: None,
        alerter: CheckAlerter::from(alerter),
        alerter_in: None,
        sites: Some(sites.into()),
      },
    };

    Ok(outage)
  }
}

#[async_trait]
impl ApiMapper for Vec<db::SiteOutage> {
  type Output = Vec<api::SiteOutage>;

  async fn map(self, pool: &Pool<MySql>) -> Result<Self::Output> {
    let outages = stream::iter(self)
      .then(async move |outage| {
        if let Ok(mut conn) = pool.acquire().await.context("could not retrieve database connection") {
          match db::Check::by_id(&mut conn, outage.check_id).await {
            Ok(check) => match check.spec(&mut conn).await {
              Ok(spec) => {
                let group = check.group(&mut conn).await;
                let alerter = check.alerter(&mut conn).await;
                let sites = check.sites(&mut conn).await.unwrap_or_default();

                let outage = api::SiteOutage {
                  outage,
                  check: api::Check {
                    check,
                    status: None,
                    spec,
                    group: CheckGroup::from(group),
                    group_in: None,
                    alerter: CheckAlerter::from(alerter),
                    alerter_in: None,
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
