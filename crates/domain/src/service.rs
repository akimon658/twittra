use crate::{
    model::{MessageListItem, Stamp, User},
    repository::Repository,
    traq_client::TraqClient,
};
use anyhow::Result;
use std::{fmt::Debug, sync::Arc};
use uuid::Uuid;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait TimelineService: Debug + Send + Sync {
    async fn get_recommended_messages(&self) -> Result<Vec<MessageListItem>>;
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait TraqService: Debug + Send + Sync {
    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<User>;
    async fn get_user_icon(&self, user_id: &Uuid) -> Result<(Vec<u8>, String)>;
    async fn get_stamp_by_id(&self, stamp_id: &Uuid) -> Result<Stamp>;
    async fn get_stamp_image(&self, stamp_id: &Uuid) -> Result<(Vec<u8>, String)>;
    async fn get_stamps(&self) -> Result<Vec<Stamp>>;
    async fn search_stamps(&self, name: &str) -> Result<Vec<Stamp>>;
    async fn add_message_stamp(
        &self,
        user_id: &Uuid,
        message_id: &Uuid,
        stamp_id: &Uuid,
        count: i32,
    ) -> Result<()>;
    async fn remove_message_stamp(
        &self,
        user_id: &Uuid,
        message_id: &Uuid,
        stamp_id: &Uuid,
    ) -> Result<()>;
}

/// Service for timeline-related operations.
#[derive(Clone, Debug)]
pub struct TimelineServiceImpl {
    repo: Repository,
}

impl TimelineServiceImpl {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait]
impl TimelineService for TimelineServiceImpl {
    async fn get_recommended_messages(&self) -> Result<Vec<MessageListItem>> {
        let messages = self.repo.message.find_recent_messages().await?;
        Ok(messages)
    }
}

/// Handles general data fetching from traQ.
/// It utilizes the repository as a cache and fetches data from traQ only when necessary.
/// Twittra's unique features such as recommendations are not handled here.
#[derive(Clone, Debug)]
pub struct TraqServiceImpl {
    repo: Repository,
    traq_client: Arc<dyn TraqClient>,
}

impl TraqServiceImpl {
    pub fn new(repo: Repository, traq_client: Arc<dyn TraqClient>) -> Self {
        Self { repo, traq_client }
    }
}

#[async_trait::async_trait]
impl TraqService for TraqServiceImpl {
    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<User> {
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

    async fn get_user_icon(&self, user_id: &Uuid) -> Result<(Vec<u8>, String)> {
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

    async fn get_stamp_by_id(&self, stamp_id: &Uuid) -> Result<Stamp> {
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

    async fn get_stamp_image(&self, stamp_id: &Uuid) -> Result<(Vec<u8>, String)> {
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

    async fn get_stamps(&self) -> Result<Vec<Stamp>> {
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

    async fn search_stamps(&self, name: &str) -> Result<Vec<Stamp>> {
        let stamps = TraqService::get_stamps(self).await?;
        let filtered = stamps
            .into_iter()
            .filter(|s| s.name.contains(name))
            .collect();
        Ok(filtered)
    }

    async fn add_message_stamp(
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

    async fn remove_message_stamp(
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
    use crate::repository::{MockMessageRepository, MockStampRepository, MockUserRepository};
    use crate::test_factories::{
        MessageListItemBuilder, RepositoryBuilder, StampBuilder, UserBuilder,
    };
    use crate::traq_client::MockTraqClient;

    // =============================================================================
    // TimelineService Tests
    // =============================================================================

    #[tokio::test]
    async fn timeline_get_recommended_messages_success() {
        let mut mock_message_repo = MockMessageRepository::new();
        let message = MessageListItemBuilder::new().build();
        let messages = vec![message.clone()];

        mock_message_repo
            .expect_find_recent_messages()
            .times(1)
            .returning(move || Ok(messages.clone()));

        let repo = RepositoryBuilder::new().message(mock_message_repo).build();

        let service = TimelineServiceImpl::new(repo);
        let result = service.get_recommended_messages().await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, message.id);
        assert_eq!(result[0].content, message.content);
    }

    #[tokio::test]
    async fn timeline_get_recommended_messages_empty() {
        let mut mock_message_repo = MockMessageRepository::new();

        mock_message_repo
            .expect_find_recent_messages()
            .times(1)
            .returning(|| Ok(vec![]));

        let repo = RepositoryBuilder::new().message(mock_message_repo).build();

        let service = TimelineServiceImpl::new(repo);
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

        let repo = RepositoryBuilder::new().message(mock_message_repo).build();

        let service = TimelineServiceImpl::new(repo);
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
        let user = UserBuilder::new().id(user_id).build();
        let user_for_mock = user.clone();

        mock_user_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(user_id))
            .times(1)
            .returning(move |_| Ok(Some(user_for_mock.clone())));

        let repo = RepositoryBuilder::new().user(mock_user_repo).build();

        let mock_client = MockTraqClient::new();
        let service = TraqServiceImpl::new(repo, Arc::new(mock_client));

        let result = service.get_user_by_id(&user_id).await.unwrap();

        // Verify we got the cached user
        assert_eq!(result.id, user_id);
        assert_eq!(result.handle, user.handle);
        assert_eq!(result.display_name, user.display_name);
        // TraqClient should NOT have been called (cache hit)
    }

    #[tokio::test]
    async fn traq_get_user_by_id_cache_miss() {
        let user_id = Uuid::now_v7();
        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_client = MockTraqClient::new();
        let user = UserBuilder::new().id(user_id).build();

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
            .returning(move |_, _| Ok(user.clone()));

        let repo = RepositoryBuilder::new().user(mock_user_repo).build();

        let service = TraqServiceImpl::new(repo, Arc::new(mock_client));
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

        let repo = RepositoryBuilder::new().user(mock_user_repo).build();

        let mock_client = MockTraqClient::new();
        let service = TraqServiceImpl::new(repo, Arc::new(mock_client));

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
            StampBuilder::new().name("golang").build(),
            StampBuilder::new().name("rust").build(),
            StampBuilder::new().name("go_fast").build(),
        ];

        mock_client
            .expect_get_stamps()
            .times(1)
            .returning(move |_| Ok(stamps.clone()));

        mock_stamp_repo
            .expect_save_batch()
            .times(1)
            .returning(|_| Ok(()));

        let repo = RepositoryBuilder::new()
            .stamp(mock_stamp_repo)
            .user(mock_user_repo)
            .build();

        let service = TraqServiceImpl::new(repo, Arc::new(mock_client));
        let result = service.search_stamps("go").await.unwrap();

        // Should return "golang" and "go_fast" but not "rust"
        assert_eq!(result.len(), 2);
        let names: Vec<_> = result.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"golang"));
        assert!(names.contains(&"go_fast"));
        assert!(!names.contains(&"rust"));
    }
}
