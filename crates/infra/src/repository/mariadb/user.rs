use domain::{error::RepositoryError, model::User, repository::UserRepository};
use sqlx::MySqlPool;
use uuid::Uuid;

#[derive(Debug)]
pub struct MariaDbUserRepository {
    pool: MySqlPool,
}

impl MariaDbUserRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for MariaDbUserRepository {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>, RepositoryError> {
        let user = match sqlx::query_as!(
            User,
            r#"
            SELECT id as `id: _`, handle, display_name
            FROM users
            WHERE id = ?
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(user) => Some(user),
            Err(sqlx::Error::RowNotFound) => None,
            Err(e) => return Err(RepositoryError::Database(e.to_string())),
        };

        Ok(user)
    }

    async fn find_random_valid_token(&self) -> Result<Option<String>, RepositoryError> {
        let rows_count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM user_tokens
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

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
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(Some(record.access_token))
    }

    async fn find_token_by_user_id(
        &self,
        user_id: &Uuid,
    ) -> Result<Option<String>, RepositoryError> {
        let record = match sqlx::query!(
            r#"
            SELECT access_token
            FROM user_tokens
            WHERE user_id = ?
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(record) => Some(record),
            Err(sqlx::Error::RowNotFound) => None,
            Err(e) => return Err(RepositoryError::Database(e.to_string())),
        };

        Ok(record.map(|r| r.access_token))
    }

    async fn save_token(&self, user_id: &Uuid, access_token: &str) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            INSERT INTO user_tokens (user_id, access_token)
            VALUES (?, ?)
            ON DUPLICATE KEY UPDATE access_token = VALUE(access_token)
            "#,
            user_id,
            access_token
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn save(&self, user: &User) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            INSERT INTO users (id, handle, display_name)
            VALUES (?, ?, ?)
            ON DUPLICATE KEY UPDATE display_name = VALUE(display_name)
            "#,
            user.id,
            user.handle,
            user.display_name,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_save_and_find_user(pool: sqlx::MySqlPool) {
        let repo = MariaDbUserRepository::new(pool);

        let user = User {
            id: Uuid::now_v7(),
            handle: "test_user".to_string(),
            display_name: "Test User".to_string(),
        };

        // Save user
        repo.save(&user).await.unwrap();

        // Find user
        let found = repo.find_by_id(&user.id).await.unwrap();

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, user.id);
        assert_eq!(found.handle, user.handle);
        assert_eq!(found.display_name, user.display_name);
    }

    #[sqlx::test]
    async fn test_find_nonexistent_user(pool: sqlx::MySqlPool) {
        let repo = MariaDbUserRepository::new(pool);

        let result = repo.find_by_id(&Uuid::now_v7()).await.unwrap();

        assert!(result.is_none());
    }

    #[sqlx::test]
    async fn test_save_and_find_token(pool: sqlx::MySqlPool) {
        let repo = MariaDbUserRepository::new(pool);

        let user_id = Uuid::now_v7();
        let token = "test_access_token_12345";

        // Create user first (FK constraint)
        let user = User {
            id: user_id,
            handle: "test_user".to_string(),
            display_name: "Test User".to_string(),
        };
        repo.save(&user).await.unwrap();

        // Save token
        repo.save_token(&user_id, token).await.unwrap();

        // Find token
        let found = repo.find_token_by_user_id(&user_id).await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap(), token);
    }

    #[sqlx::test]
    async fn test_update_token(pool: sqlx::MySqlPool) {
        let repo = MariaDbUserRepository::new(pool);

        let user_id = Uuid::now_v7();
        let token1 = "token_v1";
        let token2 = "token_v2";

        // Create user first (FK constraint)
        let user = User {
            id: user_id,
            handle: "test_user".to_string(),
            display_name: "Test User".to_string(),
        };
        repo.save(&user).await.unwrap();

        // Save original token
        repo.save_token(&user_id, token1).await.unwrap();

        // Update token
        repo.save_token(&user_id, token2).await.unwrap();

        // Verify update
        let found = repo.find_token_by_user_id(&user_id).await.unwrap();
        assert_eq!(found.unwrap(), token2);
    }

    #[sqlx::test]
    async fn test_find_random_valid_token_empty(pool: sqlx::MySqlPool) {
        let repo = MariaDbUserRepository::new(pool);

        let result = repo.find_random_valid_token().await.unwrap();

        assert!(result.is_none());
    }

    #[sqlx::test]
    async fn test_find_random_valid_token(pool: sqlx::MySqlPool) {
        let repo = MariaDbUserRepository::new(pool);

        // Create users first (FK constraint)
        let user_ids = [Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7()];
        for (i, user_id) in user_ids.iter().enumerate() {
            let user = User {
                id: *user_id,
                handle: format!("test_user_{}", i),
                display_name: format!("Test User {}", i),
            };
            repo.save(&user).await.unwrap();
        }

        // Save some tokens
        repo.save_token(&user_ids[0], "token1").await.unwrap();
        repo.save_token(&user_ids[1], "token2").await.unwrap();
        repo.save_token(&user_ids[2], "token3").await.unwrap();

        // Find random token
        let result = repo.find_random_valid_token().await.unwrap();

        assert!(result.is_some());
        let token = result.unwrap();
        assert!(["token1", "token2", "token3"].contains(&token.as_str()));
    }
}
