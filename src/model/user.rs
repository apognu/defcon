use anyhow::Result;
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::rngs::OsRng;
use sqlx::{FromRow, MySqlConnection};

use crate::api::error::{AppError, Shortable};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct User {
  #[serde(skip)]
  pub id: u64,
  #[serde(skip_deserializing)]
  pub uuid: String,
  pub email: String,
  #[serde(skip_serializing)]
  pub password: String,
  pub name: String,
}

impl User {
  pub async fn list(conn: &mut MySqlConnection) -> Result<Vec<User>> {
    let users = sqlx::query_as::<_, User>(
      "
        SELECT id, uuid, email, password, name
        FROM users
      ",
    )
    .fetch_all(&mut *conn)
    .await
    .short()?;

    Ok(users)
  }

  pub async fn by_uuid(conn: &mut MySqlConnection, uuid: &str) -> Result<User> {
    let user = sqlx::query_as::<_, User>(
      "
        SELECT id, uuid, email, password, name
        FROM users
        WHERE uuid = ?
      ",
    )
    .bind(uuid)
    .fetch_one(&mut *conn)
    .await
    .short()?;

    Ok(user)
  }

  pub async fn by_email(conn: &mut MySqlConnection, email: &str) -> Result<User> {
    let user = sqlx::query_as::<_, User>(
      "
        SELECT id, uuid, email, password, name
        FROM users
        WHERE email = ?
      ",
    )
    .bind(email)
    .fetch_one(&mut *conn)
    .await
    .short()?;

    Ok(user)
  }

  pub async fn insert(&self, conn: &mut MySqlConnection) -> Result<User> {
    let salt = SaltString::generate(&mut OsRng);
    let argon = Argon2::default();
    let hash = argon.hash_password(self.password.as_bytes(), &salt).map_err(|_| AppError::ServerError)?;

    sqlx::query(
      "
        INSERT INTO users (uuid, email, password, name)
        VALUES ( ?, ?, ?, ? )
      ",
    )
    .bind(&self.uuid)
    .bind(&self.email)
    .bind(&hash.to_string())
    .bind(&self.name)
    .execute(&mut *conn)
    .await
    .short()?;

    let user = User::by_uuid(&mut *conn, &self.uuid).await?;

    Ok(user)
  }

  pub async fn update_password(&self, conn: &mut MySqlConnection, password: &str) -> Result<()> {
    let salt = SaltString::generate(&mut OsRng);
    let argon = Argon2::default();
    let hash = argon.hash_password(password.as_bytes(), &salt).map_err(|_| AppError::ServerError)?;

    sqlx::query(
      "
        UPDATE users
        SET password = ?
        WHERE email = ?
      ",
    )
    .bind(hash.to_string())
    .bind(&self.email)
    .execute(&mut *conn)
    .await?;

    Ok(())
  }

  pub async fn check_password(&self, password: &str) -> Result<()> {
    let hash = PasswordHash::new(&self.password).map_err(|_| AppError::ServerError)?;
    let argon = Argon2::default();

    argon.verify_password(password.as_bytes(), &hash).map_err(|_| AppError::InvalidCredentials)?;

    Ok(())
  }
}
