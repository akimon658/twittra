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

    async fn update_reactions(
        &self,
        tx: &mut Transaction<'_, MySql>,
        message_ids: &[Uuid],
        reactions: &[(Uuid, Reaction)],
    ) -> Result<(), RepositoryError> {
        if message_ids.is_empty() {
            return Ok(());
        }

        let mut query_builder = QueryBuilder::new("DELETE FROM reactions WHERE message_id IN (");
        let mut separated = query_builder.separated(", ");
        for id in message_ids {
            separated.push_bind(id);
        }
        query_builder.push(")");

        query_builder
            .build()
            .execute(&mut **tx)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

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

        query_builder
            .build()
            .execute(&mut **tx)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }
}

#[derive(FromRow)]
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
    ) -> Result<Vec<(Uuid, OffsetDateTime, OffsetDateTime)>, RepositoryError> {
        #[derive(sqlx::FromRow)]
        struct SyncCandidateRow {
            id: Uuid,
            created_at: OffsetDateTime,
            last_crawled_at: OffsetDateTime,
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
            INSERT INTO messages (id, user_id, channel_id, content, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
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

        self.update_reactions(&mut tx, &[message.id], &reactions_data)
            .await?;

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
            "INSERT INTO messages (id, user_id, channel_id, content, created_at, updated_at) ",
        );

        query_builder.push_values(messages, |mut separated, message| {
            separated
                .push_bind(message.id)
                .push_bind(message.user_id)
                .push_bind(message.channel_id)
                .push_bind(&message.content)
                .push_bind(message.created_at)
                .push_bind(message.updated_at);
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

        let message_ids = messages.iter().map(|m| m.id).collect::<Vec<_>>();
        self.update_reactions(&mut tx, &message_ids, &reactions_data)
            .await?;

        tx.commit()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn mark_messages_as_read(
        &self,
        user_id: &Uuid,
        message_ids: &[Uuid],
    ) -> Result<(), RepositoryError> {
        if message_ids.is_empty() {
            return Ok(());
        }

        let mut query_builder =
            QueryBuilder::new("INSERT IGNORE INTO read_messages (user_id, message_id) ");

        query_builder.push_values(message_ids, |mut separated, message_id| {
            separated.push_bind(user_id).push_bind(message_id);
        });

        query_builder
            .build()
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn find_top_reacted_messages(
        &self,
        user_id: &Uuid,
        limit: i64,
    ) -> Result<Vec<MessageListItem>, RepositoryError> {
        let messages: Vec<MessageRow> = sqlx::query_as!(
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
            FROM messages m
            LEFT JOIN users u ON m.user_id = u.id
            LEFT JOIN reactions r ON m.id = r.message_id
            WHERE m.created_at > DATE_SUB(NOW(), INTERVAL 7 DAY)
              AND m.user_id != ?
              AND m.id NOT IN (SELECT message_id FROM read_messages WHERE user_id = ?)
            GROUP BY m.id
            ORDER BY (COUNT(r.user_id) / POW((TIMESTAMPDIFF(HOUR, m.created_at, NOW()) + 2), 1.8)) DESC
            LIMIT ?
            "#,
            user_id,
            user_id,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        self.hydrate_messages(messages).await
    }

    async fn find_messages_by_author_allowlist(
        &self,
        author_ids: &[Uuid],
        limit: i64,
        user_id: &Uuid,
    ) -> Result<Vec<MessageListItem>, RepositoryError> {
        if author_ids.is_empty() {
            return Ok(vec![]);
        }

        let mut query_builder = QueryBuilder::new(
            r#"
            SELECT
                m.id,
                m.user_id,
                m.channel_id,
                m.content,
                m.created_at,
                m.updated_at,
                u.handle AS user_handle,
                u.display_name AS user_display_name
            FROM messages m
            LEFT JOIN users u ON m.user_id = u.id
            WHERE m.created_at > DATE_SUB(NOW(), INTERVAL 30 DAY)
            "#,
        );

        query_builder.push(" AND m.user_id IN (");
        let mut separated = query_builder.separated(", ");

        for id in author_ids {
            separated.push_bind(id);
        }

        query_builder.push(") ");
        query_builder
            .push(" AND m.id NOT IN (SELECT message_id FROM read_messages WHERE user_id = ");
        query_builder.push_bind(user_id);
        query_builder.push(") ");
        query_builder.push(" ORDER BY m.created_at DESC LIMIT ");
        query_builder.push_bind(limit);

        let messages = query_builder
            .build_query_as()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        self.hydrate_messages(messages).await
    }

    async fn find_messages_by_channel_allowlist(
        &self,
        channel_ids: &[Uuid],
        limit: i64,
        user_id: &Uuid,
    ) -> Result<Vec<MessageListItem>, RepositoryError> {
        if channel_ids.is_empty() {
            return Ok(vec![]);
        }

        let mut query_builder = QueryBuilder::new(
            r#"
            SELECT
                m.id,
                m.user_id,
                m.channel_id,
                m.content,
                m.created_at,
                m.updated_at,
                u.handle AS user_handle,
                u.display_name AS user_display_name
            FROM messages m
            LEFT JOIN users u ON m.user_id = u.id
            WHERE m.created_at > DATE_SUB(NOW(), INTERVAL 30 DAY)
            "#,
        );

        query_builder.push(" AND m.channel_id IN (");
        let mut separated = query_builder.separated(", ");
        for id in channel_ids {
            separated.push_bind(id);
        }
        query_builder.push(") ");

        query_builder
            .push(" AND m.id NOT IN (SELECT message_id FROM read_messages WHERE user_id = ");
        query_builder.push_bind(user_id);
        query_builder.push(") ");
        query_builder.push(" AND m.user_id != ");
        query_builder.push_bind(user_id);

        query_builder.push(" ORDER BY m.created_at DESC LIMIT ");
        query_builder.push_bind(limit);

        let messages = query_builder
            .build_query_as()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        self.hydrate_messages(messages).await
    }
}

impl MariaDbMessageRepository {
    async fn hydrate_messages(
        &self,
        messages: Vec<MessageRow>,
    ) -> Result<Vec<MessageListItem>, RepositoryError> {
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

    #[cfg(test)]
    pub async fn find_all_messages_for_test(
        &self,
        user_id: &Uuid,
    ) -> Result<Vec<MessageListItem>, RepositoryError> {
        let messages: Vec<MessageRow> = sqlx::query_as!(
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
            FROM messages m
            LEFT JOIN users u ON m.user_id = u.id
            WHERE
                m.user_id != ?
                AND m.id NOT IN (
                    SELECT message_id
                    FROM read_messages
                    WHERE user_id = ?
                )
            ORDER BY m.created_at DESC
            "#,
            user_id,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(format!("could not fetch messages: {}", e)))?;

        self.hydrate_messages(messages).await
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

        let user_id = UUIDv4.fake();
        let messages = repo.find_all_messages_for_test(&user_id).await.unwrap();

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

        let user_id = UUIDv4.fake();

        let messages = repo.find_all_messages_for_test(&user_id).await.unwrap();

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

        let viewer_id = UUIDv4.fake();

        // Verify reaction exists
        let messages = repo.find_all_messages_for_test(&viewer_id).await.unwrap();
        assert_eq!(messages[0].reactions.len(), 1);

        // Remove reaction
        repo.remove_reaction(&message_id, &stamp_id, &user_id)
            .await
            .unwrap();

        // Verify reaction is removed
        let messages = repo.find_all_messages_for_test(&viewer_id).await.unwrap();
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

        let user_id = UUIDv4.fake();

        let saved_messages = repo.find_all_messages_for_test(&user_id).await.unwrap();
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
    }

    #[sqlx::test]
    async fn test_save_updates_last_crawled_at(pool: sqlx::MySqlPool) {
        let repo = MariaDbMessageRepository::new(pool);

        let recent_time = fake_recent_datetime();
        let message = MessageBuilder::new().created_at(recent_time).build();
        repo.save(&message).await.unwrap();

        let first_candidates = repo.find_sync_candidates().await.unwrap();
        let first_crawled_at = first_candidates[0].2;

        sleep(Duration::from_millis(100)).await;

        repo.save(&message).await.unwrap();

        let second_candidates = repo.find_sync_candidates().await.unwrap();
        let second_crawled_at = second_candidates[0].2;

        assert!(second_crawled_at > first_crawled_at);
    }
    #[sqlx::test]
    async fn test_save_adds_new_reactions(pool: sqlx::MySqlPool) {
        let repo = MariaDbMessageRepository::new(pool);

        let message_id = UUIDv4.fake();
        let reaction1 = ReactionBuilder::new().stamp_count(1).build();

        // 1. Initial save with no reactions
        let message = MessageBuilder::new().id(message_id).build();
        repo.save(&message).await.unwrap();

        let saved = repo.find_by_id(&message_id).await.unwrap().unwrap();
        assert!(saved.reactions.is_empty());

        // 2. Update: Add reaction1
        let message = MessageBuilder::new()
            .id(message_id)
            .reactions(vec![reaction1.clone()])
            .build();
        repo.save(&message).await.unwrap();

        let saved = repo.find_by_id(&message_id).await.unwrap().unwrap();
        assert_eq!(saved.reactions.len(), 1);
        assert_eq!(saved.reactions[0].stamp_id, reaction1.stamp_id);
    }

    #[sqlx::test]
    async fn test_save_updates_reaction_counts(pool: sqlx::MySqlPool) {
        let repo = MariaDbMessageRepository::new(pool);

        let message_id = UUIDv4.fake();
        let mut reaction = ReactionBuilder::new().stamp_count(1).build();

        // 1. Initial save with count 1
        let message = MessageBuilder::new()
            .id(message_id)
            .reactions(vec![reaction.clone()])
            .build();
        repo.save(&message).await.unwrap();

        // 2. Update: count 2
        reaction.stamp_count = 2;
        let message = MessageBuilder::new()
            .id(message_id)
            .reactions(vec![reaction.clone()])
            .build();
        repo.save(&message).await.unwrap();

        let saved = repo.find_by_id(&message_id).await.unwrap().unwrap();
        assert_eq!(saved.reactions.len(), 1);
        assert_eq!(saved.reactions[0].stamp_count, 2);
    }

    #[sqlx::test]
    async fn test_save_removes_missing_reactions(pool: sqlx::MySqlPool) {
        let repo = MariaDbMessageRepository::new(pool);

        let message_id = UUIDv4.fake();
        let reaction1 = ReactionBuilder::new().stamp_count(1).build();
        let reaction2 = ReactionBuilder::new().stamp_count(1).build();

        // 1. Initial save with 2 reactions
        let message = MessageBuilder::new()
            .id(message_id)
            .reactions(vec![reaction1.clone(), reaction2.clone()])
            .build();
        repo.save(&message).await.unwrap();

        let saved = repo.find_by_id(&message_id).await.unwrap().unwrap();
        assert_eq!(saved.reactions.len(), 2);

        // 2. Update: Remove reaction2
        let message = MessageBuilder::new()
            .id(message_id)
            .reactions(vec![reaction1.clone()])
            .build();
        repo.save(&message).await.unwrap();

        let saved = repo.find_by_id(&message_id).await.unwrap().unwrap();
        assert_eq!(saved.reactions.len(), 1);
        assert_eq!(saved.reactions[0].stamp_id, reaction1.stamp_id);
    }

    #[sqlx::test]
    async fn test_find_top_reacted_messages(pool: sqlx::MySqlPool) {
        let repo = MariaDbMessageRepository::new(pool);
        let message = MessageBuilder::new()
            .created_at(OffsetDateTime::now_utc() - Duration::from_secs(3600))
            .build();
        repo.save(&message).await.unwrap();

        let user_id = UUIDv4.fake();
        let result = repo.find_top_reacted_messages(&user_id, 10).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, message.id);
    }

    #[sqlx::test]
    async fn test_find_messages_by_author_allowlist(pool: sqlx::MySqlPool) {
        let repo = MariaDbMessageRepository::new(pool);
        let user_id = UUIDv4.fake();
        let message = MessageBuilder::new()
            .user_id(user_id)
            .created_at(OffsetDateTime::now_utc() - Duration::from_secs(60))
            .build();
        repo.save(&message).await.unwrap();

        let viewer_id = UUIDv4.fake();

        let result = repo
            .find_messages_by_author_allowlist(&[user_id], 10, &viewer_id)
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, message.id);
    }

    #[sqlx::test]
    async fn test_find_messages_by_channel_allowlist(pool: sqlx::MySqlPool) {
        let repo = MariaDbMessageRepository::new(pool);
        let channel_id = UUIDv4.fake();
        let message = MessageBuilder::new()
            .channel_id(channel_id)
            .created_at(OffsetDateTime::now_utc() - Duration::from_secs(60))
            .build();
        repo.save(&message).await.unwrap();

        let viewer_id = UUIDv4.fake();

        let result = repo
            .find_messages_by_channel_allowlist(&[channel_id], 10, &viewer_id)
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, message.id);
    }
}
