use std::{
  collections::HashMap,
  ops::{Deref, DerefMut},
  sync::Arc,
  time::{Duration, Instant},
};

use tokio::sync::RwLock;

#[derive(Debug)]
pub enum Delay {
  Infinite,
  Until(Instant),
}

#[derive(Debug, Clone)]
pub struct Inhibitor(pub Arc<RwLock<HashMap<String, Delay>>>);

impl Default for Inhibitor {
  fn default() -> Inhibitor {
    Inhibitor::new()
  }
}

impl Deref for Inhibitor {
  type Target = Arc<RwLock<HashMap<String, Delay>>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Inhibitor {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl Inhibitor {
  pub fn new() -> Inhibitor {
    Inhibitor(Arc::new(RwLock::new(HashMap::new())))
  }

  pub async fn inhibit(&mut self, site: &str, check: &str) {
    self.write().await.insert(format!("{site}-{check}"), Delay::Infinite);
  }

  pub async fn inhibit_for(&mut self, site: &str, check: &str, duration: Duration) {
    if let Some(instant) = Instant::now().checked_add(duration) {
      self.write().await.insert(format!("{site}-{check}"), Delay::Until(instant));
    }
  }

  pub async fn release(&mut self, site: &str, check: &str) {
    self.write().await.remove(&format!("{site}-{check}"));
  }

  pub async fn inhibited(&self, site: &str, check: &str) -> bool {
    match self.read().await.get(&format!("{site}-{check}")) {
      Some(Delay::Infinite) => true,
      Some(Delay::Until(delay)) if &Instant::now() < delay => true,
      _ => false,
    }
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use super::Inhibitor;
  use crate::config::CONTROLLER_ID;

  #[tokio::test]
  async fn default_uninhibited() {
    let inhibitor = Inhibitor::new();

    assert_eq!(inhibitor.inhibited(CONTROLLER_ID, "test").await, false);
  }

  #[tokio::test]
  async fn inhibit_forever() {
    let mut inhibitor = Inhibitor::new();
    inhibitor.inhibit(CONTROLLER_ID, "test").await;

    assert_eq!(inhibitor.inhibited(CONTROLLER_ID, "test").await, true);
  }

  #[tokio::test]
  async fn inhibit_for_two_seconds() {
    let mut inhibitor = Inhibitor::new();

    inhibitor.inhibit_for(CONTROLLER_ID, "test", Duration::from_secs(1)).await;
    assert_eq!(inhibitor.inhibited(CONTROLLER_ID, "test").await, true);

    tokio::time::sleep(Duration::from_secs(1)).await;
    assert_eq!(inhibitor.inhibited(CONTROLLER_ID, "test").await, false);
  }

  #[tokio::test]
  async fn release() {
    let mut inhibitor = Inhibitor::new();
    inhibitor.inhibit(CONTROLLER_ID, "test").await;

    inhibitor.release(CONTROLLER_ID, "test").await;
    assert_eq!(inhibitor.inhibited(CONTROLLER_ID, "test").await, false);
  }
}
