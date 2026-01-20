use std::collections::HashMap;

use domain::{
    error::RepositoryError,
    model::{Message, MessageListItem, Reaction, User},
    repository::MessageRepository,
};
use sqlx::{MySql, MySqlPool, QueryBuilder, Transaction, prelude::FromRow};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug)]
pub struct MariaDbMessageRepository {
    pool: MySqlPool,
}

impl MariaDbMessageRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    async fn save_reactions(
        &self,
        tx: &mut Transaction<'_, MySql>,
        reactions: &[(Uuid, Reaction)],
    ) -> Result<(), RepositoryError> {
        if reactions.is_empty() {
            return Ok(());
        }

        let mut query_builder = QueryBuilder::new(
            "INSERT INTO reactions (message_id, stamp_id, user_id, stamp_count) ",
        );

        query_builder.push_values(reactions, |mut separated, (msg_id, reaction)| {
            separated
                .push_bind(msg_id)
                .push_bind(reaction.stamp_id)
                .push_bind(reaction.user_id)
                .push_bind(reaction.stamp_count);
        });

        query_builder.push(" ON DUPLICATE KEY UPDATE stamp_count=VALUE(stamp_count)");

        query_builder
            .build()
            .execute(&mut **tx)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }
}

struct MessageRow {
    id: Uuid,
    user_id: Uuid,
    channel_id: Uuid,
    content: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,

    user_handle: Option<String>,
    user_display_name: Option<String>,
}

#[derive(FromRow)]
struct ReactionRow {
    message_id: Uuid,
    stamp_id: Uuid,
    user_id: Uuid,
    stamp_count: i32,
}

impl From<ReactionRow> for Reaction {
    fn from(row: ReactionRow) -> Self {
        Reaction {
            stamp_id: row.stamp_id,
            user_id: row.user_id,
            stamp_count: row.stamp_count,
        }
    }
}

struct MessageRowWithReactions(MessageRow, Vec<ReactionRow>);

impl From<MessageRowWithReactions> for MessageListItem {
    fn from(value: MessageRowWithReactions) -> Self {
        let (row, reactions) = (value.0, value.1);

        MessageListItem {
            id: row.id,
            user_id: row.user_id,
            user: match (row.user_handle, row.user_display_name) {
                (Some(handle), Some(display_name)) => Some(User {
                    id: row.user_id,
                    handle,
                    display_name,
                }),
                _ => None,
            },
            channel_id: row.channel_id,
            content: row.content,
            created_at: row.created_at,
            updated_at: row.updated_at,
            reactions: reactions.into_iter().map(Into::into).collect(),
        }
    }
}

#[async_trait::async_trait]
impl MessageRepository for MariaDbMessageRepository {
    async fn find_latest_message_time(&self) -> Result<Option<OffsetDateTime>, RepositoryError> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT MAX(created_at)
            FROM messages
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Message>, RepositoryError> {
        #[derive(sqlx::FromRow)]
        struct MessageRow {
            id: Uuid,
            user_id: Uuid,
            channel_id: Uuid,
            content: String,
            created_at: OffsetDateTime,
            updated_at: OffsetDateTime,
        }

        let message_row = sqlx::query_as!(
            MessageRow,
            r#"
            SELECT id AS `id: _`, user_id AS `user_id: _`, channel_id AS `channel_id: _`, content, created_at, updated_at
            FROM messages
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let Some(row) = message_row else {
            return Ok(None);
        };

        let reactions = sqlx::query_as!(
            ReactionRow,
            r#"
            SELECT message_id AS `message_id: _`, stamp_id AS `stamp_id: _`, user_id AS `user_id: _`, stamp_count
            FROM reactions
            WHERE message_id = ?
            "#,
            id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(Some(Message {
            id: row.id,
            user_id: row.user_id,
            channel_id: row.channel_id,
            content: row.content,
            created_at: row.created_at,
            updated_at: row.updated_at,
            reactions: reactions.into_iter().map(Into::into).collect(),
        }))
    }

    async fn find_sync_candidates(
        &self,
    ) -> Result<Vec<(Uuid, OffsetDateTime, Option<OffsetDateTime>)>, RepositoryError> {
        #[derive(sqlx::FromRow)]
        struct SyncCandidateRow {
            id: Uuid,
            created_at: OffsetDateTime,
            last_crawled_at: Option<OffsetDateTime>,
        }

        let rows = sqlx::query_as!(
            SyncCandidateRow,
            r#"
            SELECT id AS `id: _`, created_at, last_crawled_at AS `last_crawled_at: _`
            FROM messages
            WHERE created_at >= DATE_SUB(NOW(), INTERVAL 24 HOUR)
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| (row.id, row.created_at, row.last_crawled_at))
            .collect())
    }

    async fn find_recent_messages(&self) -> Result<Vec<MessageListItem>, RepositoryError> {
        let messages = sqlx::query_as!(
            MessageRow,
            r#"
            SELECT
                m.id AS `id: _`,
                m.user_id AS `user_id: _`,
                m.channel_id AS `channel_id: _`,
                m.content,
                m.created_at,
                m.updated_at,
                u.handle AS user_handle,
                u.display_name AS user_display_name
            FROM (
                SELECT id
                FROM messages
                ORDER BY created_at DESC
                LIMIT 50
            ) AS latest_messages
            JOIN messages m ON latest_messages.id = m.id
            LEFT JOIN users u ON m.user_id = u.id
            ORDER BY m.created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            RepositoryError::Database(format!("could not fetch recent messages: {}", e))
        })?;

        if messages.is_empty() {
            return Ok(vec![]);
        }

        let mut query_builder = QueryBuilder::new(
            r#"
            SELECT
                message_id,
                stamp_id,
                user_id,
                stamp_count
            FROM reactions
            WHERE message_id IN (
            "#,
        );
        let mut separated = query_builder.separated(", ");

        for msg in &messages {
            separated.push_bind(msg.id);
        }

        query_builder.push(")");

        let reactions = query_builder
            .build_query_as::<ReactionRow>()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(format!("could not fetch reactions: {}", e)))?;

        let mut message_reaction_map = HashMap::<Uuid, Vec<ReactionRow>>::new();

        for reaction in reactions {
            let entry = message_reaction_map.entry(reaction.message_id).or_default();

            entry.push(reaction);
        }

        let messages = messages
            .into_iter()
            .map(|msg| {
                let reactions = message_reaction_map.remove(&msg.id).unwrap_or_default();

                MessageListItem::from(MessageRowWithReactions(msg, reactions))
            })
            .collect();

        Ok(messages)
    }

    async fn remove_reaction(
        &self,
        message_id: &Uuid,
        stamp_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            DELETE FROM reactions
            WHERE message_id = ? AND stamp_id = ? AND user_id = ?
            "#,
            message_id,
            stamp_id,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn save(&self, message: &Message) -> Result<(), RepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        sqlx::query!(
            r#"
            INSERT INTO messages (id, user_id, channel_id, content, created_at, updated_at, last_crawled_at)
            VALUES (?, ?, ?, ?, ?, ?, NOW(6))
            ON DUPLICATE KEY UPDATE content=VALUE(content), updated_at=VALUE(updated_at), last_crawled_at=NOW(6)
            "#,
            message.id,
            message.user_id,
            message.channel_id,
            message.content,
            message.created_at,
            message.updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let reactions_data: Vec<_> = message
            .reactions
            .iter()
            .map(|r| (message.id, r.clone()))
            .collect();

        self.save_reactions(&mut tx, &reactions_data).await?;

        tx.commit()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn save_batch(&self, messages: &[Message]) -> Result<(), RepositoryError> {
        if messages.is_empty() {
            return Ok(());
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        let mut query_builder = QueryBuilder::new(
            "INSERT INTO messages (id, user_id, channel_id, content, created_at, updated_at, last_crawled_at) ",
        );

        query_builder.push_values(messages, |mut separated, message| {
            separated
                .push_bind(message.id)
                .push_bind(message.user_id)
                .push_bind(message.channel_id)
                .push_bind(&message.content)
                .push_bind(message.created_at)
                .push_bind(message.updated_at)
                .push("NOW(6)");
        });
        query_builder
            .push(" ON DUPLICATE KEY UPDATE content=VALUE(content), updated_at=VALUE(updated_at), last_crawled_at=NOW(6)");
        query_builder
            .build()
            .execute(&mut *tx)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let reactions_data = messages
            .iter()
            .flat_map(|msg| {
                msg.reactions
                    .iter()
                    .map(move |reaction| (msg.id, reaction.clone()))
            })
            .collect::<Vec<_>>();

        self.save_reactions(&mut tx, &reactions_data).await?;

        tx.commit()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::test_factories::{MessageBuilder, ReactionBuilder, fake_recent_datetime};
    use fake::{Fake, uuid::UUIDv4};
    use std::time::Duration;
    use tokio::time::sleep;

    #[sqlx::test]
    async fn test_save_and_find_message(pool: sqlx::MySqlPool) {
        let repo = MariaDbMessageRepository::new(pool);

        // Create a test message
        let message = MessageBuilder::new().build();

        // Save the message
        repo.save(&message).await.unwrap();

        // Find recent messages
        let messages = repo.find_recent_messages().await.unwrap();

        // Verify
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, message.id);
        assert_eq!(messages[0].content, message.content);
    }

    #[sqlx::test]
    async fn test_save_message_with_reactions(pool: sqlx::MySqlPool) {
        let repo = MariaDbMessageRepository::new(pool);

        let reaction = ReactionBuilder::new().build();

        let message = MessageBuilder::new()
            .reactions(vec![reaction.clone()])
            .build();

        repo.save(&message).await.unwrap();

        let messages = repo.find_recent_messages().await.unwrap();

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].reactions.len(), 1);
        assert_eq!(messages[0].reactions[0].stamp_id, reaction.stamp_id);
        assert_eq!(messages[0].reactions[0].stamp_count, reaction.stamp_count);
    }

    #[sqlx::test]
    async fn test_remove_reaction(pool: sqlx::MySqlPool) {
        let repo = MariaDbMessageRepository::new(pool);

        let message_id = UUIDv4.fake();
        let stamp_id = UUIDv4.fake();
        let user_id = UUIDv4.fake();

        let reaction = ReactionBuilder::new()
            .stamp_id(stamp_id)
            .user_id(user_id)
            .build();

        let message = MessageBuilder::new()
            .id(message_id)
            .reactions(vec![reaction])
            .build();

        // Save with reaction
        repo.save(&message).await.unwrap();

        // Verify reaction exists
        let messages = repo.find_recent_messages().await.unwrap();
        assert_eq!(messages[0].reactions.len(), 1);

        // Remove reaction
        repo.remove_reaction(&message_id, &stamp_id, &user_id)
            .await
            .unwrap();

        // Verify reaction is removed
        let messages = repo.find_recent_messages().await.unwrap();
        assert_eq!(messages[0].reactions.len(), 0);
    }

    #[sqlx::test]
    async fn test_save_batch(pool: sqlx::MySqlPool) {
        let repo = MariaDbMessageRepository::new(pool);

        let channel_id = UUIDv4.fake();
        let messages = vec![
            MessageBuilder::new().channel_id(channel_id).build(),
            MessageBuilder::new().channel_id(channel_id).build(),
        ];

        repo.save_batch(&messages).await.unwrap();

        let saved_messages = repo.find_recent_messages().await.unwrap();
        assert!(saved_messages.len() >= 2); // At least our 2 messages
    }

    #[sqlx::test]
    async fn test_find_sync_candidates_returns_recent_messages(pool: sqlx::MySqlPool) {
        let repo = MariaDbMessageRepository::new(pool);

        let recent_time = fake_recent_datetime();
        let recent_message = MessageBuilder::new()
            .created_at(recent_time - time::Duration::hours(1))
            .build();
        let old_message = MessageBuilder::new()
            .created_at(recent_time - time::Duration::hours(25))
            .build();

        repo.save(&recent_message).await.unwrap();
        repo.save(&old_message).await.unwrap();

        let candidates = repo.find_sync_candidates().await.unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].0, recent_message.id);
        assert!(candidates[0].2.is_some());
    }

    #[sqlx::test]
    async fn test_find_sync_candidates_includes_last_crawled_at(pool: sqlx::MySqlPool) {
        let repo = MariaDbMessageRepository::new(pool);

        let recent_time = fake_recent_datetime();
        let message = MessageBuilder::new().created_at(recent_time).build();
        repo.save(&message).await.unwrap();

        let candidates = repo.find_sync_candidates().await.unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].0, message.id);
        assert_eq!(candidates[0].1, message.created_at);
        assert!(candidates[0].2.is_some());
    }

    #[sqlx::test]
    async fn test_save_updates_last_crawled_at(pool: sqlx::MySqlPool) {
        let repo = MariaDbMessageRepository::new(pool);

        let recent_time = fake_recent_datetime();
        let message = MessageBuilder::new().created_at(recent_time).build();
        repo.save(&message).await.unwrap();

        let first_candidates = repo.find_sync_candidates().await.unwrap();
        let first_crawled_at = first_candidates[0].2.unwrap();

        sleep(Duration::from_millis(100)).await;

        repo.save(&message).await.unwrap();

        let second_candidates = repo.find_sync_candidates().await.unwrap();
        let second_crawled_at = second_candidates[0].2.unwrap();

        assert!(second_crawled_at > first_crawled_at);
    }
}
