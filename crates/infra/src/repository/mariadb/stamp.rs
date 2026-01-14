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
    use crate::test_utils::get_test_infra;

    async fn create_test_repo() -> Result<MariaDbStampRepository> {
        let infra = get_test_infra().await?;
        let pool = infra.create_test_database("stamp_repository").await?;
        Ok(MariaDbStampRepository::new(pool))
    }

    #[tokio::test]
    async fn test_save_and_find_stamp() {
        let repo = create_test_repo().await.unwrap();

        let stamp = Stamp {
            id: Uuid::new_v4(),
            name: "test_stamp".to_string(),
        };

        // Save stamp
        repo.save(&stamp).await.unwrap();

        // Find stamp
        let found = repo.find_by_id(&stamp.id).await.unwrap();

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, stamp.id);
        assert_eq!(found.name, stamp.name);
    }

    #[tokio::test]
    async fn test_find_nonexistent_stamp() {
        let repo = create_test_repo().await.unwrap();

        let result = repo.find_by_id(&Uuid::new_v4()).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_save_batch() {
        let repo = create_test_repo().await.unwrap();

        let stamps = vec![
            Stamp {
                id: Uuid::new_v4(),
                name: "stamp1".to_string(),
            },
            Stamp {
                id: Uuid::new_v4(),
                name: "stamp2".to_string(),
            },
            Stamp {
                id: Uuid::new_v4(),
                name: "stamp3".to_string(),
            },
        ];

        // Save batch
        repo.save_batch(&stamps).await.unwrap();

        // Verify all stamps were saved
        for stamp in &stamps {
            let found = repo.find_by_id(&stamp.id).await.unwrap();
            assert!(found.is_some());
            assert_eq!(found.unwrap().name, stamp.name);
        }
    }

    #[tokio::test]
    async fn test_update_stamp() {
        let repo = create_test_repo().await.unwrap();

        let stamp_id = Uuid::new_v4();
        let stamp_v1 = Stamp {
            id: stamp_id,
            name: "original_name".to_string(),
        };

        // Save original
        repo.save(&stamp_v1).await.unwrap();

        // Update
        let stamp_v2 = Stamp {
            id: stamp_id,
            name: "updated_name".to_string(),
        };
        repo.save(&stamp_v2).await.unwrap();

        // Verify update
        let found = repo.find_by_id(&stamp_id).await.unwrap().unwrap();
        assert_eq!(found.name, "updated_name");
    }
}
