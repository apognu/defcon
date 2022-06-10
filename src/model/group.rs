use anyhow::Result;
use sqlx::{FromRow, MySqlConnection};

use crate::api::error::Shortable;

#[derive(Debug, FromRow, Default, Clone, Serialize, Deserialize)]
pub struct Group {
  #[serde(skip)]
  pub id: u64,
  #[serde(skip_deserializing)]
  pub uuid: String,
  pub name: String,
}

impl Group {
  pub async fn all(conn: &mut MySqlConnection) -> Result<Vec<Group>> {
    let groups = sqlx::query_as::<_, Group>(
      "
        SELECT id, uuid, name
        FROM groups
      ",
    )
    .fetch_all(&mut *conn)
    .await
    .short()?;

    Ok(groups)
  }

  pub async fn by_id(conn: &mut MySqlConnection, id: u64) -> Result<Group> {
    let group = sqlx::query_as::<_, Group>(
      "
        SELECT id, uuid, name
        FROM groups
        WHERE id = ?
      ",
    )
    .bind(id)
    .fetch_one(&mut *conn)
    .await
    .short()?;

    Ok(group)
  }

  pub async fn by_uuid(conn: &mut MySqlConnection, uuid: &str) -> Result<Group> {
    let group = sqlx::query_as::<_, Group>(
      "
        SELECT id, uuid, name
        FROM groups
        WHERE uuid = ?
      ",
    )
    .bind(uuid)
    .fetch_one(&mut *conn)
    .await
    .short()?;

    Ok(group)
  }

  pub async fn insert(self, conn: &mut MySqlConnection) -> Result<Group> {
    sqlx::query(
      "
        INSERT INTO groups (uuid, name)
        VALUES (?, ?)
      ",
    )
    .bind(&self.uuid)
    .bind(self.name)
    .execute(&mut *conn)
    .await
    .short()?;

    let group = Group::by_uuid(&mut *conn, &self.uuid).await?;

    Ok(group)
  }

  pub async fn update(self, conn: &mut MySqlConnection) -> Result<Group> {
    sqlx::query(
      "
        UPDATE groups
        SET name = ?
        WHERE uuid = ?
      ",
    )
    .bind(self.name)
    .bind(&self.uuid)
    .execute(&mut *conn)
    .await
    .short()?;

    let group = Group::by_uuid(&mut *conn, &self.uuid).await?;

    Ok(group)
  }

  pub async fn delete(conn: &mut MySqlConnection, uuid: &str) -> Result<()> {
    sqlx::query(
      "
        DELETE FROM groups
        WHERE uuid = ?
      ",
    )
    .bind(uuid)
    .execute(conn)
    .await
    .short()?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use anyhow::Result;

  use super::Group;
  use crate::tests;

  #[tokio::test]
  async fn list() -> Result<()> {
    let pool = tests::db_client().await?;

    {
      let mut conn = pool.acquire().await?;

      pool.create_group(None, None, "list()").await?;

      let groups = Group::all(&mut *conn).await?;

      assert_eq!(groups.len(), 1);
      assert_eq!(&groups[0].name, "list()");
    }

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn by_id() -> Result<()> {
    let pool = tests::db_client().await?;

    {
      let mut conn = pool.acquire().await?;

      pool.create_group(None, None, "by_id()").await?;

      let group = Group::by_id(&mut *conn, 1).await?;

      assert_eq!(group.name, "by_id()");
    }

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn by_uuid() -> Result<()> {
    let pool = tests::db_client().await?;

    {
      let mut conn = pool.acquire().await?;

      pool.create_group(None, None, "by_uuid()").await?;

      let group = Group::by_uuid(&mut *conn, "dd9a531a-1b0b-4a12-bc09-e5637f916261").await?;

      assert_eq!(group.name, "by_uuid()");
    }

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn insert() -> Result<()> {
    let pool = tests::db_client().await?;

    {
      let mut conn = pool.acquire().await?;

      let group = Group {
        id: 1,
        uuid: "dd9a531a-1b0b-4a12-bc09-e5637f916261".to_string(),
        name: "insert()".to_string(),
      };

      group.insert(&mut *conn).await?;

      let group = sqlx::query_as::<_, (String,)>(r#"SELECT name FROM groups WHERE uuid = "dd9a531a-1b0b-4a12-bc09-e5637f916261""#)
        .fetch_one(&*pool)
        .await?;

      assert_eq!(&group.0, "insert()");
    }

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn update() -> Result<()> {
    let pool = tests::db_client().await?;

    {
      let mut conn = pool.acquire().await?;

      pool.create_group(None, None, "update()").await?;

      let update = Group {
        id: 1,
        uuid: "dd9a531a-1b0b-4a12-bc09-e5637f916261".to_string(),
        name: "new_update()".to_string(),
      };

      update.update(&mut *conn).await?;

      let group = sqlx::query_as::<_, (String,)>(r#"SELECT name FROM groups WHERE uuid = "dd9a531a-1b0b-4a12-bc09-e5637f916261""#)
        .fetch_one(&*pool)
        .await?;

      assert_eq!(&group.0, "new_update()");
    }

    pool.cleanup().await;

    Ok(())
  }

  #[tokio::test]
  async fn delete() -> Result<()> {
    let pool = tests::db_client().await?;

    {
      let mut conn = pool.acquire().await?;

      pool.create_group(None, None, "delete()").await?;

      Group::delete(&mut *conn, "dd9a531a-1b0b-4a12-bc09-e5637f916261").await?;

      let count = sqlx::query_as::<_, (i64,)>(r#"SELECT COUNT(*) FROM groups"#).fetch_one(&*pool).await?;
      assert_eq!(count.0, 0);
    }

    pool.cleanup().await;

    Ok(())
  }
}
