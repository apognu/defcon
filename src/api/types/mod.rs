mod alerter_kind;
mod binary;
mod check;
mod check_kind;
mod date;
mod dns_record;
mod duration;
mod query;
mod site_outage;
mod spec;

use anyhow::Result;
use sqlx::{MySql, Pool};

pub use self::{binary::*, check::*, date::*, duration::*, query::*, site_outage::*, spec::*};

#[async_trait]
pub trait ApiMapper {
  type Output;

  async fn map(self, pool: &Pool<MySql>) -> Result<Self::Output>;
}
