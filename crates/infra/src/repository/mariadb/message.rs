use std::collections::HashMap;

use anyhow::{Context, Result};
use domain::{
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
    ) -> Result<()> {
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

        query_builder.build().execute(&mut **tx).await?;

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
    async fn find_latest_message_time(&self) -> Result<Option<OffsetDateTime>> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT MAX(created_at)
            FROM messages
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_recent_messages(&self) -> Result<Vec<MessageListItem>> {
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
        .with_context(|| "could not fetch recent messages")?;

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
            .with_context(|| "could not fetch reactions")?;

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
    ) -> Result<()> {
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
        .await?;

        Ok(())
    }

    async fn save(&self, message: &Message) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"
            INSERT INTO messages (id, user_id, channel_id, content, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE content=VALUE(content), updated_at=VALUE(updated_at)
            "#,
            message.id,
            message.user_id,
            message.channel_id,
            message.content,
            message.created_at,
            message.updated_at
        )
        .execute(&mut *tx)
        .await?;

        let reactions_data: Vec<_> = message
            .reactions
            .iter()
            .map(|r| (message.id, r.clone()))
            .collect();

        self.save_reactions(&mut tx, &reactions_data).await?;

        tx.commit().await?;

        Ok(())
    }

    async fn save_batch(&self, messages: &[Message]) -> Result<()> {
        if messages.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;
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
            .push(" ON DUPLICATE KEY UPDATE content=VALUE(content), updated_at=VALUE(updated_at)");
        query_builder.build().execute(&mut *tx).await?;

        let reactions_data = messages
            .iter()
            .flat_map(|msg| {
                msg.reactions
                    .iter()
                    .map(move |reaction| (msg.id, reaction.clone()))
            })
            .collect::<Vec<_>>();

        self.save_reactions(&mut tx, &reactions_data).await?;

        tx.commit().await?;

        Ok(())
    }
}
