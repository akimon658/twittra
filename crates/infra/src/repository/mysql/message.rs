use anyhow::Result;
use domain::{model::Message, repository::MessageRepository};
use sqlx::{MySqlPool, QueryBuilder};
use time::PrimitiveDateTime;

#[derive(Debug)]
pub struct MySqlMessageRepository {
    pool: MySqlPool,
}

impl MySqlMessageRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl MessageRepository for MySqlMessageRepository {
    async fn find_latest_message_time(&self) -> Result<Option<PrimitiveDateTime>> {
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
