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

struct UserIdRecord {
    user_id: Uuid,
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

    async fn find_frequently_stamped_users_by(
        &self,
        user_id: &Uuid,
        limit: i64,
    ) -> Result<Vec<Uuid>, RepositoryError> {
        let records = sqlx::query_as!(
            UserIdRecord,
            r#"
            SELECT m.user_id AS `user_id: _`
            FROM reactions r
            JOIN messages m ON r.message_id = m.id
            WHERE r.user_id = ?
            GROUP BY m.user_id
            ORDER BY COUNT(*) DESC
            LIMIT ?
            "#,
            user_id,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(records.into_iter().map(|r| r.user_id).collect())
    }

    async fn find_similar_users(
        &self,
        user_id: &Uuid,
        limit: i64,
    ) -> Result<Vec<Uuid>, RepositoryError> {
        let records = sqlx::query_as!(
            UserIdRecord,
            r#"
            SELECT r2.user_id AS `user_id: _`
            FROM reactions r1
            JOIN reactions r2 ON r1.message_id = r2.message_id
            WHERE r1.user_id = ? AND r2.user_id != ?
            GROUP BY r2.user_id
            ORDER BY COUNT(*) DESC
            LIMIT ?
            "#,
            user_id,
            user_id,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(records.into_iter().map(|r| r.user_id).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::test_factories::{MessageBuilder, ReactionBuilder, UserBuilder};
    use fake::{Fake, uuid::UUIDv4};

    #[sqlx::test]
    async fn test_save_and_find_user(pool: sqlx::MySqlPool) {
        let repo = MariaDbUserRepository::new(pool);

        let user = UserBuilder::new().build();

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

        let result = repo.find_by_id(&UUIDv4.fake()).await.unwrap();

        assert!(result.is_none());
    }

    #[sqlx::test]
    async fn test_save_and_find_token(pool: sqlx::MySqlPool) {
        let repo = MariaDbUserRepository::new(pool);

        let user_id = UUIDv4.fake();
        let token = "test_access_token_12345";

        // Create user first (FK constraint)
        let user = UserBuilder::new().id(user_id).build();
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

        let user_id = UUIDv4.fake();
        let token1 = "token_v1";
        let token2 = "token_v2";

        // Create user first (FK constraint)
        let user = UserBuilder::new().id(user_id).build();
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
        let user_ids: Vec<Uuid> = (0..3).map(|_| UUIDv4.fake()).collect();
        for user_id in user_ids.iter() {
            let user = UserBuilder::new().id(*user_id).build();
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

    #[sqlx::test]
    async fn test_find_frequently_stamped_users_by(pool: sqlx::MySqlPool) {
        use crate::repository::mariadb::message::MariaDbMessageRepository;
        use domain::repository::MessageRepository;

        let user_repo = MariaDbUserRepository::new(pool.clone());
        let message_repo = MariaDbMessageRepository::new(pool.clone());

        let me = UUIDv4.fake();
        let target_user_1 = UUIDv4.fake(); // Most frequent
        let target_user_2 = UUIDv4.fake(); // Less frequent

        // User 1: 3 reactions from me
        for _ in 0..3 {
            let msg = MessageBuilder::new().user_id(target_user_1).build();
            let reaction = ReactionBuilder::new().user_id(me).build();
            let msg_with_reaction = MessageBuilder::new()
                .id(msg.id)
                .user_id(msg.user_id)
                .reactions(vec![reaction])
                .build();
            message_repo.save(&msg_with_reaction).await.unwrap();
        }

        // User 2: 1 reaction from me
        for _ in 0..1 {
            let msg = MessageBuilder::new().user_id(target_user_2).build();
            let reaction = ReactionBuilder::new().user_id(me).build();
            let msg_with_reaction = MessageBuilder::new()
                .id(msg.id)
                .user_id(msg.user_id)
                .reactions(vec![reaction])
                .build();
            message_repo.save(&msg_with_reaction).await.unwrap();
        }

        // Other's reaction (should be ignored)
        let other = UUIDv4.fake();
        let msg = MessageBuilder::new().user_id(target_user_2).build();
        let reaction = ReactionBuilder::new().user_id(other).build();
        let msg_with_reaction = MessageBuilder::new()
            .id(msg.id)
            .user_id(msg.user_id)
            .reactions(vec![reaction])
            .build();
        message_repo.save(&msg_with_reaction).await.unwrap();

        let stamped_users = user_repo
            .find_frequently_stamped_users_by(&me, 10)
            .await
            .unwrap();

        assert_eq!(stamped_users.len(), 2);
        assert_eq!(stamped_users[0], target_user_1);
        assert_eq!(stamped_users[1], target_user_2);
    }

    #[sqlx::test]
    async fn test_find_similar_users(pool: sqlx::MySqlPool) {
        use crate::repository::mariadb::message::MariaDbMessageRepository;
        use domain::repository::MessageRepository;

        let user_repo = MariaDbUserRepository::new(pool.clone());
        let message_repo = MariaDbMessageRepository::new(pool.clone());

        let me = UUIDv4.fake();
        let similar_user_1 = UUIDv4.fake(); // Most similar (reacted to 2 same msgs)
        let similar_user_2 = UUIDv4.fake(); // Less similar (reacted to 1 same msg)
        let other_user = UUIDv4.fake(); // Not similar (reacted to different msg)

        // Msg 1: Me and Similar 1 reacted
        let msg1 = MessageBuilder::new().build();
        let reaction_me = ReactionBuilder::new().user_id(me).build();
        let reaction_s1 = ReactionBuilder::new().user_id(similar_user_1).build();
        message_repo
            .save(
                &MessageBuilder::new()
                    .id(msg1.id)
                    .reactions(vec![reaction_me.clone(), reaction_s1.clone()])
                    .build(),
            )
            .await
            .unwrap();

        // Msg 2: Me and Similar 1 and Similar 2 reacted
        let msg2 = MessageBuilder::new().build();
        let reaction_s2 = ReactionBuilder::new().user_id(similar_user_2).build();
        message_repo
            .save(
                &MessageBuilder::new()
                    .id(msg2.id)
                    .reactions(vec![
                        reaction_me.clone(),
                        reaction_s1.clone(),
                        reaction_s2.clone(),
                    ])
                    .build(),
            )
            .await
            .unwrap();

        // Msg 3: Only Other reacted
        let msg3 = MessageBuilder::new().build();
        let reaction_other = ReactionBuilder::new().user_id(other_user).build();
        message_repo
            .save(
                &MessageBuilder::new()
                    .id(msg3.id)
                    .reactions(vec![reaction_other])
                    .build(),
            )
            .await
            .unwrap();

        let similar_users = user_repo.find_similar_users(&me, 10).await.unwrap();

        assert_eq!(similar_users.len(), 2);
        assert_eq!(similar_users[0], similar_user_1); // 2 co-occurrences
        assert_eq!(similar_users[1], similar_user_2); // 1 co-occurrence
    }
}
