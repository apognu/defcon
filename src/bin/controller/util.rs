use std::{env, sync::Arc};

use anyhow::{Context, Result};
use defcon::{config::Config, model::User};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sqlx::{MySql, Pool};
use uuid::Uuid;

pub async fn process(_config: Arc<Config>, pool: Pool<MySql>) -> Result<bool> {
  match env::args().nth(1).as_deref() {
    Some("create-admin") => create_admin(pool).await,

    Some(cmd) => Err(anyhow!("unknown command '{cmd}'")),
    None => Ok(false),
  }
}

async fn create_admin(pool: Pool<MySql>) -> Result<bool> {
  let mut conn = pool.acquire().await.unwrap();
  let email = env::args().nth(2).context("user email should be provided")?;
  let name = env::args().nth(3).context("user name should be provided")?;
  let password: String = thread_rng().sample_iter(&Alphanumeric).take(16).map(char::from).collect();

  let user = User {
    id: 0,
    uuid: Uuid::new_v4().to_string(),
    email: email.clone(),
    password: password.clone(),
    name,
  };

  user.insert(&mut *conn).await.context("could not create user")?;

  println!("Admin user '{email}' was created with password '{password}'...");

  Ok(true)
}
