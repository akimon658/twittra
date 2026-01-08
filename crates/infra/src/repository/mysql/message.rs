use anyhow::Result;
use domain::{
    model::{Message, MessageListItem, User},
    repository::MessageRepository,
};
use sqlx::{MySqlPool, QueryBuilder};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug)]
pub struct MySqlMessageRepository {
    pool: MySqlPool,
}

impl MySqlMessageRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
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

impl From<MessageRow> for MessageListItem {
    fn from(row: MessageRow) -> Self {
        let user = match (row.user_handle, row.user_display_name) {
            (Some(handle), Some(display_name)) => Some(User {
                id: row.user_id,
                handle,
                display_name,
            }),
            _ => None,
        };

        MessageListItem {
            id: row.id,
            user_id: row.user_id,
            user,
            channel_id: row.channel_id,
            content: row.content,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait::async_trait]
impl MessageRepository for MySqlMessageRepository {
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
        .await?;
        let messages = messages.into_iter().map(MessageListItem::from).collect();

        Ok(messages)
    }

    async fn save_batch(&self, messages: &[Message]) -> Result<()> {
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
        query_builder.push(
            " ON DUPLICATE KEY UPDATE content=VALUES(content), updated_at=VALUES(updated_at)",
        );
        query_builder.build().execute(&self.pool).await?;

        Ok(())
    }
}
