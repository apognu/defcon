use std::ops::{Deref, DerefMut};

use sqlx::{MySql, Pool};

#[derive(Clone)]
pub struct TestConnection(pub Pool<MySql>, pub String);

impl Deref for TestConnection {
  type Target = Pool<MySql>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for TestConnection {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl TestConnection {
  pub async fn cleanup(self) {
    sqlx::query(&format!("DROP DATABASE {}", self.1)).execute(&self.0).await.unwrap();
  }
}
