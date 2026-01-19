use crate::{error::DomainError, repository::Repository, traq_client::TraqClient};
use ::time::{Duration, OffsetDateTime};
use std::{sync::Arc, time::Duration as StdDuration};
use tokio::time;

/// Fetches new messages from traQ every 30 seconds and saves them to the repository.
pub struct MessageCrawler {
    client: Arc<dyn TraqClient>,
    repo: Repository,
}

impl MessageCrawler {
    pub fn new(client: Arc<dyn TraqClient>, repo: Repository) -> Self {
        Self { client, repo }
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

        self.refresh_messages(&token).await?;

        Ok(())
    }

    async fn refresh_messages(&self, token: &str) -> Result<(), DomainError> {
        let candidates = self.repo.message.find_sync_candidates().await?;
        let now = OffsetDateTime::now_utc();

        for (message_id, created_at, last_crawled_at) in candidates {
            if !should_refresh(created_at, last_crawled_at, now) {
                continue;
            }

            match self.client.get_message(token, &message_id).await {
                Ok(message) => {
                    self.repo.message.save(&message).await?;
                    tracing::debug!("Refreshed message {}", message_id);
                }
                Err(e) => {
                    tracing::warn!("Failed to refresh message {}: {:?}", message_id, e);
                }
            }
        }

        Ok(())
    }
}

fn should_refresh(
    created_at: OffsetDateTime,
    last_crawled_at: Option<OffsetDateTime>,
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

    match last_crawled_at {
        Some(last_crawled) => now - last_crawled >= interval,
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::{MockMessageRepository, MockUserRepository};
    use crate::test_factories::{MessageBuilder, RepositoryBuilder};
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

        let repo = RepositoryBuilder::new()
            .message(mock_message_repo)
            .user(mock_user_repo)
            .build();

        let crawler = MessageCrawler::new(Arc::new(mock_client), repo);
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

        let crawler = MessageCrawler::new(Arc::new(mock_client), repo);
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

        let crawler = MessageCrawler::new(Arc::new(MockTraqClient::new()), repo);
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
            .returning(move || Ok(vec![(message_id, created_at, Some(last_crawled_at))]));

        let refreshed_message = MessageBuilder::new().id(message_id).build();
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

        let crawler = MessageCrawler::new(Arc::new(mock_client), repo);
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
            .returning(move || Ok(vec![(message_id, created_at, Some(last_crawled_at))]));

        let repo = RepositoryBuilder::new()
            .message(mock_message_repo)
            .user(mock_user_repo)
            .build();

        let crawler = MessageCrawler::new(Arc::new(mock_client), repo);
        let result = crawler.crawl().await;

        assert!(result.is_ok());
    }

    #[test]
    fn should_refresh_returns_true_for_never_crawled_message() {
        let now = OffsetDateTime::now_utc();
        let created_at = now - Duration::hours(1);

        assert!(should_refresh(created_at, None, now));
    }

    #[test]
    fn should_refresh_recent_message_within_interval() {
        let now = OffsetDateTime::now_utc();
        let created_at = now - Duration::hours(2);
        let last_crawled_at = now - Duration::minutes(2);

        assert!(should_refresh(created_at, Some(last_crawled_at), now));
    }

    #[test]
    fn should_refresh_recent_message_not_within_interval() {
        let now = OffsetDateTime::now_utc();
        let created_at = now - Duration::hours(2);
        let last_crawled_at = now - Duration::seconds(30);

        assert!(!should_refresh(created_at, Some(last_crawled_at), now));
    }

    #[test]
    fn should_refresh_medium_age_message_within_interval() {
        let now = OffsetDateTime::now_utc();
        let created_at = now - Duration::hours(6);
        let last_crawled_at = now - Duration::minutes(11);

        assert!(should_refresh(created_at, Some(last_crawled_at), now));
    }

    #[test]
    fn should_refresh_medium_age_message_not_within_interval() {
        let now = OffsetDateTime::now_utc();
        let created_at = now - Duration::hours(6);
        let last_crawled_at = now - Duration::minutes(5);

        assert!(!should_refresh(created_at, Some(last_crawled_at), now));
    }

    #[test]
    fn should_refresh_old_message_within_interval() {
        let now = OffsetDateTime::now_utc();
        let created_at = now - Duration::hours(18);
        let last_crawled_at = now - Duration::minutes(31);

        assert!(should_refresh(created_at, Some(last_crawled_at), now));
    }

    #[test]
    fn should_refresh_old_message_not_within_interval() {
        let now = OffsetDateTime::now_utc();
        let created_at = now - Duration::hours(18);
        let last_crawled_at = now - Duration::minutes(20);

        assert!(!should_refresh(created_at, Some(last_crawled_at), now));
    }
}
