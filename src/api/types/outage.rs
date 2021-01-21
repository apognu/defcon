use anyhow::{Context, Result};
use futures::{stream, StreamExt};
use serde::Deserialize;
use sqlx::{MySql, Pool};

use crate::{
  api::types::{self as api, ApiMapper},
  model as db,
};

#[derive(Debug, Serialize)]
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
    let alerter = check.alerter(&mut *conn).await;

    let outage = api::Outage {
      outage: self,
      check: api::Check {
        check,
        spec,
        alerter: alerter.map(|alerter| alerter.uuid),
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
                let alerter = check.alerter(&mut *conn).await;

                let outage = api::Outage {
                  outage,
                  check: api::Check {
                    check,
                    spec,
                    alerter: alerter.map(|alerter| alerter.uuid),
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
