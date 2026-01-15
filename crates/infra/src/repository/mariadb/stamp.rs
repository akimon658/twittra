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

    async fn save_batch(&self, stamps: &[Stamp]) -> Result<()> {
        if stamps.is_empty() {
            return Ok(());
        }

        let mut query_builder = sqlx::QueryBuilder::new("INSERT INTO stamps (id, name) ");

        query_builder.push_values(stamps, |mut separated, stamp| {
            separated.push_bind(stamp.id).push_bind(&stamp.name);
        });

        query_builder.push(" ON DUPLICATE KEY UPDATE name = VALUE(name)");

        query_builder.build().execute(&self.pool).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_save_and_find_stamp(pool: sqlx::MySqlPool) -> anyhow::Result<()> {
        let repo = MariaDbStampRepository::new(pool);

        let stamp = Stamp {
            id: Uuid::now_v7(),
            name: "test_stamp".to_string(),
        };

        // Save stamp
        repo.save(&stamp).await?;

        // Find stamp
        let found = repo.find_by_id(&stamp.id).await?;

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, stamp.id);
        assert_eq!(found.name, stamp.name);
        
        Ok(())
    }

    #[sqlx::test]
    async fn test_find_nonexistent_stamp(pool: sqlx::MySqlPool) -> anyhow::Result<()> {
        let repo = MariaDbStampRepository::new(pool);

        let result = repo.find_by_id(&Uuid::now_v7()).await?;

        assert!(result.is_none());
        
        Ok(())
    }

    #[sqlx::test]
    async fn test_save_batch(pool: sqlx::MySqlPool) -> anyhow::Result<()> {
        let repo = MariaDbStampRepository::new(pool);

        let stamps = vec![
            Stamp {
                id: Uuid::now_v7(),
                name: "stamp1".to_string(),
            },
            Stamp {
                id: Uuid::now_v7(),
                name: "stamp2".to_string(),
            },
            Stamp {
                id: Uuid::now_v7(),
                name: "stamp3".to_string(),
            },
        ];

        // Save batch
        repo.save_batch(&stamps).await?;

        // Verify all stamps were saved
        for stamp in &stamps {
            let found = repo.find_by_id(&stamp.id).await?;
            assert!(found.is_some());
            assert_eq!(found.unwrap().name, stamp.name);
        }
        
        Ok(())
    }

    #[sqlx::test]
    async fn test_update_stamp(pool: sqlx::MySqlPool) -> anyhow::Result<()> {
        let repo = MariaDbStampRepository::new(pool);

        let stamp_id = Uuid::now_v7();
        let stamp_v1 = Stamp {
            id: stamp_id,
            name: "original_name".to_string(),
        };

        // Save original
        repo.save(&stamp_v1).await?;

        // Update
        let stamp_v2 = Stamp {
            id: stamp_id,
            name: "updated_name".to_string(),
        };
        repo.save(&stamp_v2).await?;

        // Verify update
        let found = repo.find_by_id(&stamp_id).await?.unwrap();
        assert_eq!(found.name, "updated_name");
        
        Ok(())
    }
}
