use anyhow::{Context, Result};
use futures::{stream, StreamExt};
use sqlx::{MySql, Pool};

use crate::{
  api::types::{self as api, ApiMapper},
  model as db,
};

#[derive(Debug, Serialize)]
pub struct Timeline {
  #[serde(flatten)]
  pub timeline: db::Timeline,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub author: Option<db::User>,
}

#[async_trait]
impl ApiMapper for db::Timeline {
  type Output = api::Timeline;

  async fn map(self, pool: &Pool<MySql>) -> Result<Self::Output> {
    let mut conn = pool.acquire().await.context("could not retrieve database connection")?;

    match self.user_id {
      Some(user_id) => {
        let user = db::User::by_id(&mut conn, user_id).await?;

        Ok(api::Timeline { timeline: self, author: Some(user) })
      }

      None => Ok(api::Timeline { timeline: self, author: None }),
    }
  }
}

#[async_trait]
impl ApiMapper for Vec<db::Timeline> {
  type Output = Vec<api::Timeline>;

  async fn map(self, pool: &Pool<MySql>) -> Result<Self::Output> {
    let timelines = stream::iter(self)
      .then(async move |timeline| {
        if let Ok(mut conn) = pool.acquire().await.context("could not retrieve database connection") {
          match timeline.user_id {
            Some(user_id) => {
              let user = db::User::by_id(&mut conn, user_id).await.ok();

              Some(api::Timeline { timeline, author: user })
            }

            None => Some(api::Timeline { timeline, author: None }),
          }
        } else {
          None
        }
      })
      .filter_map(async move |timeline| timeline)
      .collect()
      .await;

    Ok(timelines)
  }
}
