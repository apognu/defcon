use std::ops::Deref;

use crate::model as db;

#[derive(Debug, Serialize, Deserialize)]
pub struct Sites(pub Vec<String>);

impl Deref for Sites {
  type Target = [String];

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<Vec<db::Site>> for Sites {
  fn from(sites: Vec<db::Site>) -> Sites {
    Sites(sites.into_iter().map(|site| site.slug).collect())
  }
}
