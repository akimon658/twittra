use anyhow::Result;
use domain::{model::User, repository::UserRepository};
use sqlx::MySqlPool;
use uuid::Uuid;

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
            SELECT id as `id: _`, handle
            FROM users
            WHERE id = ?
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
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
            INSERT INTO users (id, handle)
            VALUES (?, ?)
            ON DUPLICATE KEY UPDATE handle = VALUES(handle)
            "#,
            user.id,
            user.handle
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
