use std::{
  collections::HashMap,
  ops::{Deref, DerefMut},
  sync::Arc,
};

use tokio::sync::RwLock;

use crate::model::Check;

#[derive(Debug, Clone)]
pub struct Stash(pub Arc<RwLock<HashMap<String, String>>>);

impl Default for Stash {
  fn default() -> Stash {
    Stash::new()
  }
}

impl Deref for Stash {
  type Target = Arc<RwLock<HashMap<String, String>>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Stash {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl Stash {
  pub fn new() -> Stash {
    Stash(Arc::new(RwLock::new(HashMap::new())))
  }

  pub async fn stash(&mut self, check: &Check, key: &str, value: &str) {
    self.write().await.insert(format!("{}-{}", check.uuid, key), value.to_owned());
  }

  pub async fn retrieve(&self, check: &Check, key: &str) -> Option<String> {
    self.read().await.get(&format!("{}-{}", check.uuid, key)).map(ToOwned::to_owned)
  }

  pub async fn delete(&mut self, check: &Check, key: &str) {
    self.write().await.remove(&format!("{}-{}", check.uuid, key));
  }
}

#[cfg(test)]
mod tests {
  use super::Stash;
  use crate::model::Check;

  #[tokio::test]
  async fn can_create_entry() {
    let check = Check::default();
    let mut stash = Stash::new();

    assert_eq!(stash.retrieve(&check, "test").await, None);

    stash.stash(&check, "test", "helloworld").await;
    assert_eq!(stash.retrieve(&check, "test").await, Some("helloworld".to_string()));
  }

  #[tokio::test]
  async fn can_delete_entry() {
    let check = Check::default();
    let mut stash = Stash::new();
    stash.stash(&check, "test", "helloworld").await;

    stash.delete(&check, "test").await;
    assert_eq!(stash.retrieve(&check, "test").await, None);
  }
}
