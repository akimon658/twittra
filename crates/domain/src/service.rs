use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    model::{MessageListItem, Stamp, User},
    repository::Repository,
    traq_client::TraqClient,
};

/// Service for timeline-related operations.
#[derive(Clone, Debug)]
pub struct TimelineService {
    repo: Repository,
}

impl TimelineService {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub async fn get_recommended_messages(&self) -> Result<Vec<MessageListItem>> {
        let messages = self.repo.message.find_recent_messages().await?;

        Ok(messages)
    }
}

/// Handles general data fetching from traQ.
/// It utilizes the repository as a cache and fetches data from traQ only when necessary.
/// Twittra's unique features such as recommendations are not handled here.
#[derive(Clone, Debug)]
pub struct TraqService {
    repo: Repository,
    traq_client: Arc<dyn TraqClient>,
}

impl TraqService {
    pub fn new(repo: Repository, traq_client: Arc<dyn TraqClient>) -> Self {
        Self { repo, traq_client }
    }

    pub async fn get_user_by_id(&self, user_id: &Uuid) -> Result<User> {
        let user = match self.repo.user.find_by_id(user_id).await? {
            Some(user) => user,
            None => {
                let token = match self.repo.user.find_random_valid_token().await? {
                    Some(token) => token,
                    None => {
                        return Err(anyhow::anyhow!(
                            "no valid token found to fetch user from traQ"
                        ));
                    }
                };
                let user = self.traq_client.get_user(&token, user_id).await?;

                self.repo.user.save(&user).await?;

                user
            }
        };

        Ok(user)
    }

    pub async fn get_user_icon(&self, user_id: &Uuid) -> Result<(Vec<u8>, String)> {
        let token = match self.repo.user.find_random_valid_token().await? {
            Some(token) => token,
            None => {
                return Err(anyhow::anyhow!(
                    "no valid token found to fetch user icon from traQ"
                ));
            }
        };
        let icon = self.traq_client.get_user_icon(&token, user_id).await?;

        Ok(icon)
    }

    pub async fn get_stamp_image(&self, stamp_id: &Uuid) -> Result<(Vec<u8>, String)> {
        let token = match self.repo.user.find_random_valid_token().await? {
            Some(token) => token,
            None => {
                return Err(anyhow::anyhow!(
                    "no valid token found to fetch stamp image from traQ"
                ));
            }
        };
        let image = self.traq_client.get_stamp_image(&token, stamp_id).await?;

        Ok(image)
    }

    pub async fn get_stamp_by_id(&self, stamp_id: &Uuid) -> Result<Stamp> {
        let stamp = match self.repo.stamp.find_by_id(stamp_id).await? {
            Some(stamp) => stamp,
            None => {
                let token = match self.repo.user.find_random_valid_token().await? {
                    Some(token) => token,
                    None => {
                        return Err(anyhow::anyhow!(
                            "no valid token found to fetch stamp from traQ"
                        ));
                    }
                };
                let stamp = self.traq_client.get_stamp(&token, stamp_id).await?;

                self.repo.stamp.save(&stamp).await?;

                stamp
            }
        };

        Ok(stamp)
    }

    pub async fn get_stamps(&self) -> Result<Vec<Stamp>> {
        let token = match self.repo.user.find_random_valid_token().await? {
            Some(token) => token,
            None => {
                return Err(anyhow::anyhow!(
                    "no valid token found to fetch stamps from traQ"
                ));
            }
        };
        let stamps = self.traq_client.get_stamps(&token).await?;

        self.repo.stamp.save_batch(&stamps).await?;

        Ok(stamps)
    }

    pub async fn search_stamps(&self, name: &str) -> Result<Vec<Stamp>> {
        let stamps = self.get_stamps().await?;
        let filtered = stamps
            .into_iter()
            .filter(|s| s.name.contains(name))
            .collect();
        Ok(filtered)
    }

    pub async fn add_message_stamp(
        &self,
        user_id: &Uuid,
        message_id: &Uuid,
        stamp_id: &Uuid,
        count: i32,
    ) -> Result<()> {
        let token = match self.repo.user.find_token_by_user_id(user_id).await? {
            Some(token) => token,
            None => {
                return Err(anyhow::anyhow!("no valid token found for user {}", user_id));
            }
        };

        // 1. Add stamp to traQ
        self.traq_client
            .add_message_stamp(&token, message_id, stamp_id, count)
            .await?;

        // 2. Fetch updated message from traQ (to get latest reactions)
        let message = self.traq_client.get_message(&token, message_id).await?;

        // 3. Update local DB
        self.repo.message.save(&message).await?;

        Ok(())
    }

    pub async fn remove_message_stamp(
        &self,
        user_id: &Uuid,
        message_id: &Uuid,
        stamp_id: &Uuid,
    ) -> Result<()> {
        let token = match self.repo.user.find_token_by_user_id(user_id).await? {
            Some(token) => token,
            None => {
                return Err(anyhow::anyhow!("no valid token found for user {}", user_id));
            }
        };

        // 1. Remove stamp from traQ
        self.traq_client
            .remove_message_stamp(&token, message_id, stamp_id)
            .await?;

        // 2. Optimistically update local DB by directly removing the reaction
        //    traQ does not immediately reflect the removal in subsequent fetches,
        //    so we directly update the local cache here.
        self.repo
            .message
            .remove_reaction(message_id, stamp_id, user_id)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{MessageListItem, Stamp, User};
    use crate::repository::{MockMessageRepository, MockStampRepository, MockUserRepository};
    use crate::traq_client::MockTraqClient;
    use time::OffsetDateTime;
    use uuid::Uuid;

    // Helper to create test MessageListItem
    fn test_message_list_item() -> MessageListItem {
        MessageListItem {
            id: Uuid::now_v7(),
            user_id: Uuid::now_v7(),
            user: None,
            channel_id: Uuid::now_v7(),
            content: "test message".to_string(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            reactions: vec![],
        }
    }

    fn test_user(id: Uuid) -> User {
        User {
            id,
            handle: "test_user".to_string(),
            display_name: "Test User".to_string(),
        }
    }

    #[allow(dead_code)]
    fn test_stamp(id: Uuid) -> Stamp {
        Stamp {
            id,
            name: "test_stamp".to_string(),
        }
    }

    // =============================================================================
    // TimelineService Tests
    // =============================================================================

    #[tokio::test]
    async fn timeline_get_recommended_messages_success() {
        let mut mock_message_repo = MockMessageRepository::new();
        let messages = vec![test_message_list_item()];
        let messages_clone = messages.clone();

        mock_message_repo
            .expect_find_recent_messages()
            .times(1)
            .returning(move || Ok(messages_clone.clone()));

        let repo = Repository {
            message: Arc::new(mock_message_repo),
            stamp: Arc::new(MockStampRepository::new()),
            user: Arc::new(MockUserRepository::new()),
        };

        let service = TimelineService::new(repo);
        let result = service.get_recommended_messages().await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "test message");
    }

    #[tokio::test]
    async fn timeline_get_recommended_messages_empty() {
        let mut mock_message_repo = MockMessageRepository::new();

        mock_message_repo
            .expect_find_recent_messages()
            .times(1)
            .returning(|| Ok(vec![]));

        let repo = Repository {
            message: Arc::new(mock_message_repo),
            stamp: Arc::new(MockStampRepository::new()),
            user: Arc::new(MockUserRepository::new()),
        };

        let service = TimelineService::new(repo);
        let result = service.get_recommended_messages().await.unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn timeline_get_recommended_messages_error() {
        let mut mock_message_repo = MockMessageRepository::new();

        mock_message_repo
            .expect_find_recent_messages()
            .times(1)
            .returning(|| Err(anyhow::anyhow!("database error")));

        let repo = Repository {
            message: Arc::new(mock_message_repo),
            stamp: Arc::new(MockStampRepository::new()),
            user: Arc::new(MockUserRepository::new()),
        };

        let service = TimelineService::new(repo);
        let result = service.get_recommended_messages().await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "database error");
    }

    // =============================================================================
    // TraqService Tests
    // =============================================================================

    #[tokio::test]
    async fn traq_get_user_by_id_cache_hit() {
        let user_id = Uuid::now_v7();
        let mut mock_user_repo = MockUserRepository::new();
        let user = test_user(user_id);
        let user_clone = user.clone();

        mock_user_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(user_id))
            .times(1)
            .returning(move |_| Ok(Some(user_clone.clone())));

        let repo = Repository {
            message: Arc::new(MockMessageRepository::new()),
            stamp: Arc::new(MockStampRepository::new()),
            user: Arc::new(mock_user_repo),
        };

        let mock_client = MockTraqClient::new();
        let service = TraqService::new(repo, Arc::new(mock_client));

        let result = service.get_user_by_id(&user_id).await.unwrap();

        // Verify we got the cached user
        assert_eq!(result.id, user_id);
        assert_eq!(result.handle, "test_user");
        // TraqClient should NOT have been called (cache hit)
    }

    #[tokio::test]
    async fn traq_get_user_by_id_cache_miss() {
        let user_id = Uuid::now_v7();
        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_client = MockTraqClient::new();
        let user = test_user(user_id);
        let user_clone = user.clone();

        // Cache miss
        mock_user_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(user_id))
            .times(1)
            .returning(|_| Ok(None));

        // Need token to fetch from traQ
        mock_user_repo
            .expect_find_random_valid_token()
            .times(1)
            .returning(|| Ok(Some("test_token".to_string())));

        // Save fetched user
        mock_user_repo.expect_save().times(1).returning(|_| Ok(()));

        // Fetch from traQ
        mock_client
            .expect_get_user()
            .withf(|token, _| token == "test_token")
            .times(1)
            .returning(move |_, _| Ok(user_clone.clone()));

        let repo = Repository {
            message: Arc::new(MockMessageRepository::new()),
            stamp: Arc::new(MockStampRepository::new()),
            user: Arc::new(mock_user_repo),
        };

        let service = TraqService::new(repo, Arc::new(mock_client));
        let result = service.get_user_by_id(&user_id).await.unwrap();

        assert_eq!(result.id, user_id);
    }

    #[tokio::test]
    async fn traq_get_user_by_id_no_token_error() {
        let user_id = Uuid::now_v7();
        let mut mock_user_repo = MockUserRepository::new();

        mock_user_repo
            .expect_find_by_id()
            .times(1)
            .returning(|_| Ok(None));

        mock_user_repo
            .expect_find_random_valid_token()
            .times(1)
            .returning(|| Ok(None));

        let repo = Repository {
            message: Arc::new(MockMessageRepository::new()),
            stamp: Arc::new(MockStampRepository::new()),
            user: Arc::new(mock_user_repo),
        };

        let mock_client = MockTraqClient::new();
        let service = TraqService::new(repo, Arc::new(mock_client));

        let result = service.get_user_by_id(&user_id).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("no valid token found")
        );
    }

    #[tokio::test]
    async fn traq_search_stamps_filters_correctly() {
        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_stamp_repo = MockStampRepository::new();
        let mut mock_client = MockTraqClient::new();

        mock_user_repo
            .expect_find_random_valid_token()
            .times(1)
            .returning(|| Ok(Some("test_token".to_string())));

        let stamps = vec![
            Stamp {
                id: Uuid::now_v7(),
                name: "golang".to_string(),
            },
            Stamp {
                id: Uuid::now_v7(),
                name: "rust".to_string(),
            },
            Stamp {
                id: Uuid::now_v7(),
                name: "go_fast".to_string(),
            },
        ];
        let stamps_clone = stamps.clone();

        mock_client
            .expect_get_stamps()
            .times(1)
            .returning(move |_| Ok(stamps_clone.clone()));

        mock_stamp_repo
            .expect_save_batch()
            .times(1)
            .returning(|_| Ok(()));

        let repo = Repository {
            message: Arc::new(MockMessageRepository::new()),
            stamp: Arc::new(mock_stamp_repo),
            user: Arc::new(mock_user_repo),
        };

        let service = TraqService::new(repo, Arc::new(mock_client));
        let result = service.search_stamps("go").await.unwrap();

        // Should return "golang" and "go_fast" but not "rust"
        assert_eq!(result.len(), 2);
        let names: Vec<_> = result.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"golang"));
        assert!(names.contains(&"go_fast"));
        assert!(!names.contains(&"rust"));
    }
}
