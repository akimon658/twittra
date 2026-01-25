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
        struct ChannelIdRecord {
            channel_id: Uuid,
        }

        let records = sqlx::query_as!(
            ChannelIdRecord,
            r#"
            SELECT m.channel_id AS `channel_id: _`
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
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(records.into_iter().map(|r| r.channel_id).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::test_factories::{MessageBuilder, ReactionBuilder, StampBuilder};
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

    #[sqlx::test]
    async fn test_find_frequently_stamped_channels_by(pool: sqlx::MySqlPool) {
        use crate::repository::mariadb::message::MariaDbMessageRepository;
        use domain::repository::MessageRepository;

        let stamp_repo = MariaDbStampRepository::new(pool.clone());
        let message_repo = MariaDbMessageRepository::new(pool.clone());

        let user_id = UUIDv4.fake();
        let channel_1 = UUIDv4.fake();
        let channel_2 = UUIDv4.fake();

        // Channel 1: 3 reactions
        for _ in 0..3 {
            let msg = MessageBuilder::new().channel_id(channel_1).build();
            let reaction = ReactionBuilder::new().user_id(user_id).build();
            let msg_with_reaction = MessageBuilder::new()
                .id(msg.id)
                .channel_id(msg.channel_id)
                .reactions(vec![reaction])
                .build();
            message_repo.save(&msg_with_reaction).await.unwrap();
        }

        // Channel 2: 1 reaction
        for _ in 0..1 {
            let msg = MessageBuilder::new().channel_id(channel_2).build();
            let reaction = ReactionBuilder::new().user_id(user_id).build();
            let msg_with_reaction = MessageBuilder::new()
                .id(msg.id)
                .channel_id(msg.channel_id)
                .reactions(vec![reaction])
                .build();
            message_repo.save(&msg_with_reaction).await.unwrap();
        }

        // Other user's reaction (should be ignored)
        let other_user = UUIDv4.fake();
        let msg = MessageBuilder::new().channel_id(channel_2).build();
        let reaction = ReactionBuilder::new().user_id(other_user).build();
        let msg_with_reaction = MessageBuilder::new()
            .id(msg.id)
            .channel_id(msg.channel_id)
            .reactions(vec![reaction])
            .build();
        message_repo.save(&msg_with_reaction).await.unwrap();

        let channels = stamp_repo
            .find_frequently_stamped_channels_by(&user_id, 10)
            .await
            .unwrap();

        assert_eq!(channels.len(), 2);
        assert_eq!(channels[0], channel_1); // Most frequent first
        assert_eq!(channels[1], channel_2);
    }
}
