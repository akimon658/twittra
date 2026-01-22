use crate::{
    error::DomainError, model::Message, notifier::MessageNotifier, repository::Repository,
    traq_client::TraqClient,
};
use ::time::{Duration, OffsetDateTime};
use std::{sync::Arc, time::Duration as StdDuration};
use tokio::time;

/// Fetches new messages from traQ every 30 seconds and saves them to the repository.
pub struct MessageCrawler {
    client: Arc<dyn TraqClient>,
    repo: Repository,
    notifier: Arc<dyn MessageNotifier>,
}

impl MessageCrawler {
    pub fn new(
        client: Arc<dyn TraqClient>,
        repo: Repository,
        notifier: Arc<dyn MessageNotifier>,
    ) -> Self {
        Self {
            client,
            repo,
            notifier,
        }
    }

    pub async fn run(&self) {
        loop {
            if let Err(e) = self.crawl().await {
                tracing::error!("Crawl failed: {:?}", e);
            }

            time::sleep(StdDuration::from_secs(30)).await;
        }
    }

    pub async fn crawl(&self) -> Result<(), DomainError> {
        let last_fetched_at = self
            .repo
            .message
            .find_latest_message_time()
            .await?
            .unwrap_or_else(|| OffsetDateTime::now_utc() - Duration::days(1));
        let token = match self.repo.user.find_random_valid_token().await? {
            Some(t) => t,
            None => {
                tracing::warn!("No valid token found. Skipping crawl.");

                return Ok(());
            }
        };
        let messages = self
            .client
            .fetch_messages_since(&token, last_fetched_at)
            .await?;

        self.repo.message.save_batch(&messages).await?;

        let refreshed_messages = self.refresh_messages(&token).await?;

        for message in &refreshed_messages {
            self.notifier.notify_message_updated(message).await;
        }

        Ok(())
    }

    async fn refresh_messages(&self, token: &str) -> Result<Vec<Message>, DomainError> {
        let candidates = self.repo.message.find_sync_candidates().await?;
        let now = OffsetDateTime::now_utc();
        let mut refreshed_messages = Vec::new();

        for (message_id, created_at, last_crawled_at) in candidates {
            if !should_refresh(created_at, last_crawled_at, now) {
                continue;
            }

            match self.client.get_message(token, &message_id).await {
                Ok(new_message) => {
                    let existing_message = match self.repo.message.find_by_id(&message_id).await? {
                        Some(msg) => msg,
                        None => {
                            // Since message_id is from the repo, this should not happen
                            return Err(DomainError::NoMessageForId(message_id));
                        }
                    };

                    // Always save to update last_crawled_at
                    self.repo.message.save(&new_message).await?;

                    // Only notify if the message actually changed
                    if existing_message != new_message {
                        tracing::debug!("Refreshed message {}", message_id);
                        refreshed_messages.push(new_message);
                    } else {
                        tracing::debug!("Message {} unchanged, skipping notification", message_id);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to refresh message {}: {:?}", message_id, e);
                }
            }
        }

        Ok(refreshed_messages)
    }
}

fn should_refresh(
    created_at: OffsetDateTime,
    last_crawled_at: OffsetDateTime,
    now: OffsetDateTime,
) -> bool {
    let age = now - created_at;
    let interval = if age < Duration::hours(3) {
        Duration::minutes(1)
    } else if age < Duration::hours(12) {
        Duration::minutes(10)
    } else {
        Duration::minutes(30)
    };

    now - last_crawled_at >= interval
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notifier::MockMessageNotifier;
    use crate::repository::{MockMessageRepository, MockUserRepository};
    use crate::test_factories::{MessageBuilder, ReactionBuilder, RepositoryBuilder};
    use crate::traq_client::MockTraqClient;
    use fake::{Fake, uuid::UUIDv4};
    use mockall::predicate;

    #[tokio::test]
    async fn crawl_success_with_existing_messages() {
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_client = MockTraqClient::new();

        let latest_message_time = OffsetDateTime::now_utc() - Duration::hours(1);
        let token = "test_token".to_string();
        let messages = vec![MessageBuilder::new().build()];

        // 1. Get latest message time
        mock_message_repo
            .expect_find_latest_message_time()
            .times(1)
            .returning(move || Ok(Some(latest_message_time)));

        // 2. Get valid token
        mock_user_repo
            .expect_find_random_valid_token()
            .times(1)
            .returning(move || Ok(Some(token.clone())));

        // 3. Fetch messages from traQ
        mock_client
            .expect_fetch_messages_since()
            .with(
                predicate::eq("test_token"),
                predicate::eq(latest_message_time),
            )
            .times(1)
            .returning(move |_, _| Ok(messages.clone()));

        // 4. Save messages to repository
        mock_message_repo
            .expect_save_batch()
            .times(1)
            .returning(|_| Ok(()));

        // 5. Find sync candidates (for refresh)
        mock_message_repo
            .expect_find_sync_candidates()
            .times(1)
            .returning(|| Ok(vec![]));

        // Notifier should NOT be called since there are no messages to refresh
        let mock_notifier = MockMessageNotifier::new();
        let repo = RepositoryBuilder::new()
            .message(mock_message_repo)
            .user(mock_user_repo)
            .build();

        let crawler = MessageCrawler::new(Arc::new(mock_client), repo, Arc::new(mock_notifier));
        let result = crawler.crawl().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn crawl_success_no_previous_messages_fallback() {
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_client = MockTraqClient::new();

        // 1. No latest message (returns None)
        mock_message_repo
            .expect_find_latest_message_time()
            .times(1)
            .returning(move || Ok(None));

        // 2. Get valid token
        mock_user_repo
            .expect_find_random_valid_token()
            .times(1)
            .returning(move || Ok(Some("test_token".to_string())));

        // 3. Fetch messages from traQ - should fallback to 1 day ago
        // We can't easily check exact time due to dynamic fallback, so just check call existence
        mock_client
            .expect_fetch_messages_since()
            .times(1)
            .returning(move |_, _| Ok(vec![]));

        // 4. Save batch (empty)
        mock_message_repo
            .expect_save_batch()
            .times(1)
            .returning(|_| Ok(()));

        // 5. Find sync candidates
        mock_message_repo
            .expect_find_sync_candidates()
            .times(1)
            .returning(|| Ok(vec![]));

        let repo = RepositoryBuilder::new()
            .message(mock_message_repo)
            .user(mock_user_repo)
            .build();

        let mock_notifier = MockMessageNotifier::new();

        let crawler = MessageCrawler::new(Arc::new(mock_client), repo, Arc::new(mock_notifier));
        let result = crawler.crawl().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn crawl_skips_when_no_token() {
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();

        mock_message_repo
            .expect_find_latest_message_time()
            .returning(|| Ok(None));

        // No token
        mock_user_repo
            .expect_find_random_valid_token()
            .returning(|| Ok(None));

        let repo = RepositoryBuilder::new()
            .message(mock_message_repo)
            .user(mock_user_repo)
            .build();

        let mock_notifier = MockMessageNotifier::new();

        let crawler = MessageCrawler::new(
            Arc::new(MockTraqClient::new()),
            repo,
            Arc::new(mock_notifier),
        );
        let result = crawler.crawl().await;

        // Should succeed (return Ok) but log warning and skip fetch
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn crawl_refreshes_messages_needing_update() {
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_client = MockTraqClient::new();

        let now = OffsetDateTime::now_utc();
        let message_id = UUIDv4.fake();
        let created_at = now - Duration::minutes(30);
        let last_crawled_at = now - Duration::minutes(2);

        mock_message_repo
            .expect_find_latest_message_time()
            .returning(move || Ok(Some(now)));

        mock_user_repo
            .expect_find_random_valid_token()
            .returning(|| Ok(Some("test_token".to_string())));

        mock_client
            .expect_fetch_messages_since()
            .returning(|_, _| Ok(vec![]));

        mock_message_repo.expect_save_batch().returning(|_| Ok(()));

        mock_message_repo
            .expect_find_sync_candidates()
            .times(1)
            .returning(move || Ok(vec![(message_id, created_at, last_crawled_at)]));

        let existing_message = MessageBuilder::new().id(message_id).build();
        let refreshed_message = existing_message.clone();

        // Expect find_by_id to return existing message
        mock_message_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(existing_message.clone())));

        mock_client
            .expect_get_message()
            .times(1)
            .returning(move |_, _| Ok(refreshed_message.clone()));

        mock_message_repo
            .expect_save()
            .times(1)
            .returning(|_| Ok(()));

        let repo = RepositoryBuilder::new()
            .message(mock_message_repo)
            .user(mock_user_repo)
            .build();

        // Notifier should NOT be called since message is unchanged
        let mock_notifier = MockMessageNotifier::new();

        let crawler = MessageCrawler::new(Arc::new(mock_client), repo, Arc::new(mock_notifier));
        let result = crawler.crawl().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn crawl_notifies_when_message_content_changed() {
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_client = MockTraqClient::new();

        let now = OffsetDateTime::now_utc();
        let message_id = UUIDv4.fake();
        let created_at = now - Duration::minutes(30);
        let last_crawled_at = now - Duration::minutes(2);

        mock_message_repo
            .expect_find_latest_message_time()
            .returning(move || Ok(Some(now)));

        mock_user_repo
            .expect_find_random_valid_token()
            .returning(|| Ok(Some("test_token".to_string())));

        mock_client
            .expect_fetch_messages_since()
            .returning(|_, _| Ok(vec![]));

        mock_message_repo.expect_save_batch().returning(|_| Ok(()));

        mock_message_repo
            .expect_find_sync_candidates()
            .times(1)
            .returning(move || Ok(vec![(message_id, created_at, last_crawled_at)]));

        let existing_message = MessageBuilder::new()
            .id(message_id)
            .content("old content".to_string())
            .build();
        let refreshed_message = MessageBuilder::new()
            .id(message_id)
            .content("new content".to_string())
            .build();

        mock_message_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(existing_message.clone())));

        mock_client
            .expect_get_message()
            .times(1)
            .returning(move |_, _| Ok(refreshed_message.clone()));

        mock_message_repo
            .expect_save()
            .times(1)
            .returning(|_| Ok(()));

        let repo = RepositoryBuilder::new()
            .message(mock_message_repo)
            .user(mock_user_repo)
            .build();

        let mut mock_notifier = MockMessageNotifier::new();
        mock_notifier
            .expect_notify_message_updated()
            .times(1)
            .returning(|_| ());

        let crawler = MessageCrawler::new(Arc::new(mock_client), repo, Arc::new(mock_notifier));
        let result = crawler.crawl().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn crawl_notifies_when_reactions_changed() {
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_client = MockTraqClient::new();

        let now = OffsetDateTime::now_utc();
        let message_id = UUIDv4.fake();
        let created_at = now - Duration::minutes(30);
        let last_crawled_at = now - Duration::minutes(2);

        mock_message_repo
            .expect_find_latest_message_time()
            .returning(move || Ok(Some(now)));

        mock_user_repo
            .expect_find_random_valid_token()
            .returning(|| Ok(Some("test_token".to_string())));

        mock_client
            .expect_fetch_messages_since()
            .returning(|_, _| Ok(vec![]));

        mock_message_repo.expect_save_batch().returning(|_| Ok(()));

        mock_message_repo
            .expect_find_sync_candidates()
            .times(1)
            .returning(move || Ok(vec![(message_id, created_at, last_crawled_at)]));

        let reaction1 = ReactionBuilder::new().stamp_count(1).build();
        let reaction2 = ReactionBuilder::new().stamp_count(2).build();

        let existing_message = MessageBuilder::new()
            .id(message_id)
            .reactions(vec![reaction1.clone()])
            .build();
        let refreshed_message = MessageBuilder::new()
            .id(message_id)
            .reactions(vec![reaction1, reaction2])
            .build();

        mock_message_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(existing_message.clone())));

        mock_client
            .expect_get_message()
            .times(1)
            .returning(move |_, _| Ok(refreshed_message.clone()));

        mock_message_repo
            .expect_save()
            .times(1)
            .returning(|_| Ok(()));

        let repo = RepositoryBuilder::new()
            .message(mock_message_repo)
            .user(mock_user_repo)
            .build();

        let mut mock_notifier = MockMessageNotifier::new();
        mock_notifier
            .expect_notify_message_updated()
            .times(1)
            .returning(|_| ());

        let crawler = MessageCrawler::new(Arc::new(mock_client), repo, Arc::new(mock_notifier));
        let result = crawler.crawl().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn crawl_skips_messages_not_needing_refresh() {
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_client = MockTraqClient::new();

        let now = OffsetDateTime::now_utc();
        let message_id = UUIDv4.fake();
        let created_at = now - Duration::minutes(30);
        let last_crawled_at = now - Duration::seconds(30);

        mock_message_repo
            .expect_find_latest_message_time()
            .returning(move || Ok(Some(now)));

        mock_user_repo
            .expect_find_random_valid_token()
            .returning(|| Ok(Some("test_token".to_string())));

        mock_client
            .expect_fetch_messages_since()
            .returning(|_, _| Ok(vec![]));

        mock_message_repo.expect_save_batch().returning(|_| Ok(()));

        mock_message_repo
            .expect_find_sync_candidates()
            .times(1)
            .returning(move || Ok(vec![(message_id, created_at, last_crawled_at)]));

        let repo = RepositoryBuilder::new()
            .message(mock_message_repo)
            .user(mock_user_repo)
            .build();

        let mock_notifier = MockMessageNotifier::new();

        let crawler = MessageCrawler::new(Arc::new(mock_client), repo, Arc::new(mock_notifier));
        let result = crawler.crawl().await;

        assert!(result.is_ok());
    }

    #[test]
    fn should_refresh_recent_message_within_interval() {
        let now = OffsetDateTime::now_utc();
        let created_at = now - Duration::hours(2);
        let last_crawled_at = now - Duration::minutes(2);

        assert!(should_refresh(created_at, last_crawled_at, now));
    }

    #[test]
    fn should_refresh_recent_message_not_within_interval() {
        let now = OffsetDateTime::now_utc();
        let created_at = now - Duration::hours(2);
        let last_crawled_at = now - Duration::seconds(30);

        assert!(!should_refresh(created_at, last_crawled_at, now));
    }

    #[test]
    fn should_refresh_medium_age_message_within_interval() {
        let now = OffsetDateTime::now_utc();
        let created_at = now - Duration::hours(6);
        let last_crawled_at = now - Duration::minutes(11);

        assert!(should_refresh(created_at, last_crawled_at, now));
    }

    #[test]
    fn should_refresh_medium_age_message_not_within_interval() {
        let now = OffsetDateTime::now_utc();
        let created_at = now - Duration::hours(6);
        let last_crawled_at = now - Duration::minutes(5);

        assert!(!should_refresh(created_at, last_crawled_at, now));
    }

    #[test]
    fn should_refresh_old_message_within_interval() {
        let now = OffsetDateTime::now_utc();
        let created_at = now - Duration::hours(18);
        let last_crawled_at = now - Duration::minutes(31);

        assert!(should_refresh(created_at, last_crawled_at, now));
    }

    #[test]
    fn should_refresh_old_message_not_within_interval() {
        let now = OffsetDateTime::now_utc();
        let created_at = now - Duration::hours(18);
        let last_crawled_at = now - Duration::minutes(20);

        assert!(!should_refresh(created_at, last_crawled_at, now));
    }
}
