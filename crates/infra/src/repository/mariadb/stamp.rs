use domain::{error::RepositoryError, model::Stamp, repository::StampRepository};
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
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Stamp>, RepositoryError> {
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
            Err(e) => return Err(RepositoryError::Database(e.to_string())),
        };

        Ok(stamp)
    }

    async fn save(&self, stamp: &Stamp) -> Result<(), RepositoryError> {
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
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn save_batch(&self, stamps: &[Stamp]) -> Result<(), RepositoryError> {
        if stamps.is_empty() {
            return Ok(());
        }

        let mut query_builder = sqlx::QueryBuilder::new("INSERT INTO stamps (id, name) ");

        query_builder.push_values(stamps, |mut separated, stamp| {
            separated.push_bind(stamp.id).push_bind(&stamp.name);
        });

        query_builder.push(" ON DUPLICATE KEY UPDATE name = VALUE(name)");

        query_builder
            .build()
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn find_frequently_stamped_channels_by(
        &self,
        user_id: &Uuid,
        limit: i64,
    ) -> Result<Vec<Uuid>, RepositoryError> {
        let channel_ids = sqlx::query!(
            r#"
            SELECT m.channel_id
            FROM reactions r
            JOIN messages m ON r.message_id = m.id
            WHERE r.user_id = ?
            GROUP BY m.channel_id
            ORDER BY COUNT(*) DESC
            LIMIT ?
            "#,
            user_id,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?
        .into_iter()
        .map(|record| {
            if record.channel_id.len() == 16 {
                Uuid::from_slice(&record.channel_id).unwrap_or_default()
            } else {
                let s = String::from_utf8(record.channel_id).unwrap_or_default();
                Uuid::parse_str(&s).unwrap_or_default()
            }
        })
        .collect();

        Ok(channel_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::test_factories::StampBuilder;
    use fake::{Fake, uuid::UUIDv4};

    #[sqlx::test]
    async fn test_save_and_find_stamp(pool: sqlx::MySqlPool) {
        let repo = MariaDbStampRepository::new(pool);

        let stamp = StampBuilder::new().build();

        // Save stamp
        repo.save(&stamp).await.unwrap();

        // Find stamp
        let found = repo.find_by_id(&stamp.id).await.unwrap();

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, stamp.id);
        assert_eq!(found.name, stamp.name);
    }

    #[sqlx::test]
    async fn test_find_nonexistent_stamp(pool: sqlx::MySqlPool) {
        let repo = MariaDbStampRepository::new(pool);

        let result = repo.find_by_id(&UUIDv4.fake()).await.unwrap();

        assert!(result.is_none());
    }

    #[sqlx::test]
    async fn test_save_batch(pool: sqlx::MySqlPool) {
        let repo = MariaDbStampRepository::new(pool);

        let stamps = vec![
            StampBuilder::new().build(),
            StampBuilder::new().build(),
            StampBuilder::new().build(),
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

    #[sqlx::test]
    async fn test_update_stamp(pool: sqlx::MySqlPool) {
        let repo = MariaDbStampRepository::new(pool);

        let stamp_id = UUIDv4.fake();
        let stamp_v1 = StampBuilder::new()
            .id(stamp_id)
            .name("original_name")
            .build();

        // Save original
        repo.save(&stamp_v1).await.unwrap();

        // Update
        let stamp_v2 = StampBuilder::new()
            .id(stamp_id)
            .name("updated_name")
            .build();
        repo.save(&stamp_v2).await.unwrap();

        // Verify update
        let found = repo.find_by_id(&stamp_id).await.unwrap().unwrap();
        assert_eq!(found.name, "updated_name");
    }
}
