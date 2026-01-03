use anyhow::Result;
use domain::{model::User, repository::UserRepository};
use sqlx::MySqlPool;
use uuid::Uuid;

#[derive(Debug)]
pub struct MySqlUserRepository {
    pool: MySqlPool,
}

impl MySqlUserRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for MySqlUserRepository {
    async fn find_by_id(&self, id: &Uuid) -> Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id as `id: _`, handle, display_name
            FROM users
            WHERE id = ?
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_random_valid_token(&self) -> Result<Option<String>> {
        let rows_count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM user_tokens
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        if rows_count == 0 {
            return Ok(None);
        }

        let random_offset = fastrand::i64(0..rows_count);
        let record = sqlx::query!(
            r#"
            SELECT access_token
            FROM user_tokens
            LIMIT 1 OFFSET ?
            "#,
            random_offset
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(record.access_token))
    }

    async fn save_token(&self, user_id: &Uuid, access_token: &str) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO user_tokens (user_id, access_token)
            VALUES (?, ?)
            ON DUPLICATE KEY UPDATE access_token = VALUES(access_token)
            "#,
            user_id,
            access_token
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn save(&self, user: &User) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO users (id, handle, display_name)
            VALUES (?, ?, ?)
            ON DUPLICATE KEY UPDATE display_name = VALUES(display_name)
            "#,
            user.id,
            user.handle,
            user.display_name,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
