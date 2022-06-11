use anyhow::Result;
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use rand::rngs::OsRng;

use crate::tests::db::TestConnection;

impl TestConnection {
  pub async fn create_user(&self) -> Result<()> {
    let salt = SaltString::generate(&mut OsRng);
    let argon = Argon2::default();
    let hash = argon.hash_password("password".as_bytes(), &salt).unwrap();

    sqlx::query(
      "
        INSERT INTO users (id, uuid, email, password, name)
        VALUES ( 1, ?, ?, ?, ? )
      ",
    )
    .bind("7fc3989e-baea-4c7b-99a9-9210d2a3422c")
    .bind("noreply@example.com")
    .bind(hash.to_string())
    .bind("Bob User")
    .execute(&**self)
    .await?;

    Ok(())
  }
}
