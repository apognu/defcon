use std::{
  collections::HashMap,
  ops::{Deref, DerefMut},
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

#[derive(Debug)]
pub enum Delay {
  Infinite,
  Until(Instant),
}

#[derive(Debug, Clone)]
pub struct Inhibitor(pub Arc<Mutex<HashMap<String, Delay>>>);

impl Default for Inhibitor {
  fn default() -> Inhibitor {
    Inhibitor::new()
  }
}

impl Deref for Inhibitor {
  type Target = Arc<Mutex<HashMap<String, Delay>>>;

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
    Inhibitor(Arc::new(Mutex::new(HashMap::new())))
  }

  pub fn inhibit(&mut self, site: &str, check: &str) {
    if let Ok(mut inhibitor) = self.lock() {
      inhibitor.insert(format!("{}-{}", site, check), Delay::Infinite);
    }
  }

  pub fn inhibit_for(&mut self, site: &str, check: &str, duration: Duration) {
    if let Ok(mut inhibitor) = self.lock() {
      if let Some(instant) = Instant::now().checked_add(duration) {
        inhibitor.insert(format!("{}-{}", site, check), Delay::Until(instant));
      }
    }
  }

  pub fn release(&mut self, site: &str, check: &str) {
    if let Ok(mut inhibitor) = self.lock() {
      inhibitor.remove(&format!("{}-{}", site, check));
    }
  }

  pub fn inhibited(&self, site: &str, check: &str) -> bool {
    match self.lock() {
      Ok(inhibitor) => match inhibitor.get(&format!("{}-{}", site, check)) {
        Some(Delay::Infinite) => true,
        Some(Delay::Until(delay)) if &Instant::now() < delay => true,
        _ => false,
      },

      Err(_) => false,
    }
  }
}
