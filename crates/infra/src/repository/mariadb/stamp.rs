use anyhow::Result;
use domain::{model::Stamp, repository::StampRepository};
use sqlx::MySqlPool;
use uuid::Uuid;

#[derive(Debug)]
pub struct MariaDbStampRepository {
    pool: MySqlPool,
}

impl MariaDbStampRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl StampRepository for MariaDbStampRepository {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Stamp>> {
        let stamp = match sqlx::query_as!(
            Stamp,
            r#"
            SELECT id as `id: _`, name
            FROM stamps
            WHERE id = ?
            "#,
            id,
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(stamp) => Some(stamp),
            Err(sqlx::Error::RowNotFound) => None,
            Err(e) => return Err(e.into()),
        };

        Ok(stamp)
    }

    async fn save(&self, stamp: &Stamp) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO stamps (id, name)
            VALUES (?, ?)
            ON DUPLICATE KEY UPDATE name = VALUE(name)
            "#,
            stamp.id,
            stamp.name,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
