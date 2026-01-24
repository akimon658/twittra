use crate::{
    error::DomainError,
    model::{MessageListItem, Stamp, User},
    repository::Repository,
    traq_client::TraqClient,
};
use std::{cmp::Ordering, collections::HashMap, fmt::Debug, sync::Arc};
use uuid::Uuid;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait TimelineService: Debug + Send + Sync {
    async fn get_recommended_messages(
        &self,
        user_id: &Uuid,
    ) -> Result<Vec<MessageListItem>, DomainError>;
    async fn mark_messages_as_read(
        &self,
        user_id: &Uuid,
        message_ids: &[Uuid],
    ) -> Result<(), DomainError>;
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait TraqService: Debug + Send + Sync {
    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<User, DomainError>;
    async fn get_user_icon(&self, user_id: &Uuid) -> Result<(Vec<u8>, String), DomainError>;
    async fn get_stamp_by_id(&self, stamp_id: &Uuid) -> Result<Stamp, DomainError>;
    async fn get_stamp_image(&self, stamp_id: &Uuid) -> Result<(Vec<u8>, String), DomainError>;
    async fn get_stamps(&self) -> Result<Vec<Stamp>, DomainError>;
    async fn search_stamps(&self, name: &str) -> Result<Vec<Stamp>, DomainError>;
    async fn add_message_stamp(
        &self,
        user_id: &Uuid,
        message_id: &Uuid,
        stamp_id: &Uuid,
        count: i32,
    ) -> Result<(), DomainError>;
    async fn remove_message_stamp(
        &self,
        user_id: &Uuid,
        message_id: &Uuid,
        stamp_id: &Uuid,
    ) -> Result<(), DomainError>;
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
    async fn get_recommended_messages(
        &self,
        user_id: &Uuid,
    ) -> Result<Vec<MessageListItem>, DomainError> {
        // 1. Get user affinity list (people I stamp)
        let affinity_users = self
            .repo
            .user
            .find_frequently_stamped_users_by(user_id, 20)
            .await?;

        // 2. Get channel affinity list (channels I stamp in)
        let affinity_channels = self
            .repo
            .stamp
            .find_frequently_stamped_channels_by(user_id, 10)
            .await?;

        // 3. Get similar users (people who stamp same msgs)
        let similar_users = self.repo.user.find_similar_users(user_id, 20).await?;

        // 4. Fetch candidates from all sources concurrently
        // To avoid finding messages that user already read or self-authored, we pass user_id.
        let (top_reacts, affinity_author_msgs, affinity_channel_msgs, similar_user_msgs) = tokio::join!(
            self.repo
                .message
                .find_top_reacted_messages(Some(*user_id), 50),
            self.repo.message.find_messages_by_author_allowlist(
                &affinity_users,
                50,
                Some(*user_id)
            ),
            self.repo.message.find_messages_by_channel_allowlist(
                &affinity_channels,
                50,
                Some(*user_id)
            ),
            self.repo
                .message
                .find_messages_by_author_allowlist(&similar_users, 50, Some(*user_id))
        );

        let top_reacts = top_reacts?;
        let affinity_author_msgs = affinity_author_msgs?;
        let affinity_channel_msgs = affinity_channel_msgs?;
        let similar_user_msgs = similar_user_msgs?;

        // 5. Merge and Score
        // Map message_id -> (Message, Score)
        // Scores:
        // - Top Reacted: 5.0 + (50 - rank) * 0.1
        // - Affinity Author: 5.0 + (50 - rank) * 0.15
        // - Affinity Channel: 3.0 + (50 - rank) * 0.1
        // - Similar User: 5.0 + (50 - rank) * 0.1

        let mut scored_messages = HashMap::<Uuid, (MessageListItem, f64)>::new();

        let mut add_score = |msgs: Vec<MessageListItem>, base_score: f64, rank_multiplier: f64| {
            for (i, msg) in msgs.into_iter().enumerate() {
                let rank_score = (50.0 - i as f64).max(0.0) * rank_multiplier;
                let total_score = base_score + rank_score;

                scored_messages
                    .entry(msg.id)
                    .and_modify(|(_, s)| *s += total_score)
                    .or_insert((msg, total_score));
            }
        };

        add_score(top_reacts, 5.0, 0.1);
        add_score(affinity_author_msgs, 5.0, 0.15);
        add_score(affinity_channel_msgs, 3.0, 0.1);
        add_score(similar_user_msgs, 5.0, 0.1);
        let mut final_list: Vec<(MessageListItem, f64)> = scored_messages.into_values().collect();
        // Sort by score descending
        final_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

        // Return top 50
        let result = final_list.into_iter().take(50).map(|(m, _)| m).collect();

        Ok(result)
    }

    async fn mark_messages_as_read(
        &self,
        user_id: &Uuid,
        message_ids: &[Uuid],
    ) -> Result<(), DomainError> {
        self.repo
            .message
            .mark_messages_as_read(user_id, message_ids)
            .await?;
        Ok(())
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
    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<User, DomainError> {
        let user = match self.repo.user.find_by_id(user_id).await? {
            Some(user) => user,
            None => {
                let token = match self.repo.user.find_random_valid_token().await? {
                    Some(token) => token,
                    None => {
                        return Err(DomainError::NoTokenForUserFetch);
                    }
                };
                let user = self.traq_client.get_user(&token, user_id).await?;
                self.repo.user.save(&user).await?;
                user
            }
        };
        Ok(user)
    }

    async fn get_user_icon(&self, user_id: &Uuid) -> Result<(Vec<u8>, String), DomainError> {
        let token = match self.repo.user.find_random_valid_token().await? {
            Some(token) => token,
            None => {
                return Err(DomainError::NoTokenForUserIcon);
            }
        };
        let icon = self.traq_client.get_user_icon(&token, user_id).await?;
        Ok(icon)
    }

    async fn get_stamp_by_id(&self, stamp_id: &Uuid) -> Result<Stamp, DomainError> {
        let stamp = match self.repo.stamp.find_by_id(stamp_id).await? {
            Some(stamp) => stamp,
            None => {
                let token = match self.repo.user.find_random_valid_token().await? {
                    Some(token) => token,
                    None => {
                        return Err(DomainError::NoTokenForStampFetch);
                    }
                };
                let stamp = self.traq_client.get_stamp(&token, stamp_id).await?;
                self.repo.stamp.save(&stamp).await?;
                stamp
            }
        };
        Ok(stamp)
    }

    async fn get_stamp_image(&self, stamp_id: &Uuid) -> Result<(Vec<u8>, String), DomainError> {
        let token = match self.repo.user.find_random_valid_token().await? {
            Some(token) => token,
            None => {
                return Err(DomainError::NoTokenForStampImage);
            }
        };
        let image = self.traq_client.get_stamp_image(&token, stamp_id).await?;
        Ok(image)
    }

    async fn get_stamps(&self) -> Result<Vec<Stamp>, DomainError> {
        let token = match self.repo.user.find_random_valid_token().await? {
            Some(token) => token,
            None => {
                return Err(DomainError::NoTokenForStampsList);
            }
        };
        let stamps = self.traq_client.get_stamps(&token).await?;
        self.repo.stamp.save_batch(&stamps).await?;
        Ok(stamps)
    }

    async fn search_stamps(&self, name: &str) -> Result<Vec<Stamp>, DomainError> {
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
    ) -> Result<(), DomainError> {
        let token = match self.repo.user.find_token_by_user_id(user_id).await? {
            Some(token) => token,
            None => {
                return Err(DomainError::NoTokenForUser(*user_id));
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
    ) -> Result<(), DomainError> {
        let token = match self.repo.user.find_token_by_user_id(user_id).await? {
            Some(token) => token,
            None => {
                return Err(DomainError::NoTokenForUser(*user_id));
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
    use crate::{
        error::RepositoryError,
        repository::{MockMessageRepository, MockStampRepository, MockUserRepository},
        test_factories::{MessageListItemBuilder, RepositoryBuilder, StampBuilder, UserBuilder},
        traq_client::MockTraqClient,
    };
    use fake::{Fake, uuid::UUIDv4};
    use mockall::predicate;

    #[tokio::test]
    async fn timeline_get_recommended_messages_success() {
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_stamp_repo = MockStampRepository::new();
        let message = MessageListItemBuilder::new().build();
        let messages = vec![message.clone()];

        // 1. Affinity / Similar users setup
        mock_user_repo
            .expect_find_frequently_stamped_users_by()
            .with(predicate::eq(message.user_id), predicate::eq(20))
            .returning(|_, _| Ok(vec![]));
        mock_stamp_repo
            .expect_find_frequently_stamped_channels_by()
            .with(predicate::eq(message.user_id), predicate::eq(10))
            .returning(|_, _| Ok(vec![]));
        mock_user_repo
            .expect_find_similar_users()
            .with(predicate::eq(message.user_id), predicate::eq(20))
            .returning(|_, _| Ok(vec![]));

        // 2. Mock setup for remaining fetches
        mock_message_repo
            .expect_find_messages_by_author_allowlist()
            .returning(|_, _, _| Ok(vec![]));
        mock_message_repo
            .expect_find_messages_by_channel_allowlist()
            .returning(|_, _, _| Ok(vec![]));

        // 3. Recommendation fetches
        mock_message_repo
            .expect_find_top_reacted_messages()
            .returning(move |_, _| Ok(messages.clone()));

        let repo = RepositoryBuilder::new()
            .message(mock_message_repo)
            .user(mock_user_repo)
            .stamp(mock_stamp_repo)
            .build();
        let service = TimelineServiceImpl::new(repo);
        let result = service
            .get_recommended_messages(&message.user_id)
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, message.id);
        assert_eq!(result[0].content, message.content);
    }

    #[tokio::test]
    async fn timeline_get_recommended_messages_empty() {
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_stamp_repo = MockStampRepository::new();

        let user_id = UUIDv4.fake();

        // Mocks returning empty/defaults
        mock_user_repo
            .expect_find_frequently_stamped_users_by()
            .returning(|_, _| Ok(vec![]));
        mock_stamp_repo
            .expect_find_frequently_stamped_channels_by()
            .returning(|_, _| Ok(vec![]));
        mock_user_repo
            .expect_find_similar_users()
            .returning(|_, _| Ok(vec![]));
        mock_message_repo
            .expect_find_messages_by_author_allowlist()
            .returning(|_, _, _| Ok(vec![]));
        mock_message_repo
            .expect_find_messages_by_channel_allowlist()
            .returning(|_, _, _| Ok(vec![]));

        mock_message_repo
            .expect_find_top_reacted_messages()
            .returning(|_, _| Ok(vec![]));

        let repo = RepositoryBuilder::new()
            .message(mock_message_repo)
            .user(mock_user_repo)
            .stamp(mock_stamp_repo)
            .build();
        let service = TimelineServiceImpl::new(repo);
        let result = service.get_recommended_messages(&user_id).await.unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn timeline_get_recommended_messages_error() {
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_stamp_repo = MockStampRepository::new();

        let user_id = UUIDv4.fake();

        mock_user_repo
            .expect_find_frequently_stamped_users_by()
            .returning(|_, _| Ok(vec![]));
        mock_stamp_repo
            .expect_find_frequently_stamped_channels_by()
            .returning(|_, _| Ok(vec![]));
        mock_user_repo
            .expect_find_similar_users()
            .returning(|_, _| Ok(vec![]));
        mock_message_repo
            .expect_find_messages_by_author_allowlist()
            .returning(|_, _, _| Ok(vec![]));
        mock_message_repo
            .expect_find_messages_by_channel_allowlist()
            .returning(|_, _, _| Ok(vec![]));

        mock_message_repo
            .expect_find_top_reacted_messages()
            .returning(|_, _| Err(RepositoryError::Database("database error".to_string())));

        let repo = RepositoryBuilder::new()
            .message(mock_message_repo)
            .user(mock_user_repo)
            .stamp(mock_stamp_repo)
            .build();
        let service = TimelineServiceImpl::new(repo);
        let result = service.get_recommended_messages(&user_id).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::Repository(_)));
    }

    #[tokio::test]
    async fn traq_get_user_by_id_cache_hit() {
        let user_id = UUIDv4.fake();
        let mut mock_user_repo = MockUserRepository::new();
        let user = UserBuilder::new().id(user_id).build();
        let user_for_mock = user.clone();

        mock_user_repo
            .expect_find_by_id()
            .with(predicate::eq(user_id))
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
        let user_id = UUIDv4.fake();
        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_client = MockTraqClient::new();
        let user = UserBuilder::new().id(user_id).build();

        // Cache miss
        mock_user_repo
            .expect_find_by_id()
            .with(predicate::eq(user_id))
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
        let user_id = UUIDv4.fake();
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
        assert_eq!(result.unwrap_err(), DomainError::NoTokenForUserFetch);
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

    #[tokio::test]
    async fn traq_remove_message_stamp_optimistically_updates_local_db() {
        let user_id = UUIDv4.fake();
        let message_id = UUIDv4.fake();
        let stamp_id = UUIDv4.fake();

        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_client = MockTraqClient::new();

        mock_user_repo
            .expect_find_token_by_user_id()
            .with(predicate::eq(user_id))
            .times(1)
            .returning(move |_| Ok(Some("test_token".to_string())));

        mock_client
            .expect_remove_message_stamp()
            .withf(|token, _, _| token == "test_token")
            .times(1)
            .returning(|_, _, _| Ok(()));

        mock_message_repo
            .expect_remove_reaction()
            .withf(move |msg_id, stp_id, usr_id| {
                *msg_id == message_id && *stp_id == stamp_id && *usr_id == user_id
            })
            .times(1)
            .returning(|_, _, _| Ok(()));

        let repo = RepositoryBuilder::new()
            .user(mock_user_repo)
            .message(mock_message_repo)
            .build();

        let service = TraqServiceImpl::new(repo, Arc::new(mock_client));
        let result = service
            .remove_message_stamp(&user_id, &message_id, &stamp_id)
            .await;

        assert!(result.is_ok());
    }
}
