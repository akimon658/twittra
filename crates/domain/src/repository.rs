use std::{fmt::Debug, sync::Arc};

use crate::error::RepositoryError;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::model::{Message, MessageListItem, Stamp, User};

#[derive(Clone, Debug)]
pub struct Repository {
    pub message: Arc<dyn MessageRepository>,
    pub stamp: Arc<dyn StampRepository>,
    pub user: Arc<dyn UserRepository>,
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait MessageRepository: Debug + Send + Sync {
    async fn find_latest_message_time(&self) -> Result<Option<OffsetDateTime>, RepositoryError>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Message>, RepositoryError>;
    async fn find_recent_messages(&self) -> Result<Vec<MessageListItem>, RepositoryError>;
    /// Returns messages that may need refreshing from traQ.
    /// Returns tuples of (message_id, created_at, last_crawled_at) for messages created within the last 24 hours.
    async fn find_sync_candidates(
        &self,
    ) -> Result<Vec<(Uuid, OffsetDateTime, Option<OffsetDateTime>)>, RepositoryError>;
    /// Removes a reaction from a message.
    /// This is used for optimistic updates when deleting a stamp.
    async fn remove_reaction(
        &self,
        message_id: &Uuid,
        stamp_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), RepositoryError>;
    /// Saves a message to the repository.
    async fn save(&self, message: &Message) -> Result<(), RepositoryError>;
    /// Saves a batch of messages to the repository.
    /// It does nothing if `messages` is empty.
    async fn save_batch(&self, messages: &[Message]) -> Result<(), RepositoryError>;
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait StampRepository: Debug + Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Stamp>, RepositoryError>;
    async fn save(&self, stamp: &Stamp) -> Result<(), RepositoryError>;
    async fn save_batch(&self, stamps: &[Stamp]) -> Result<(), RepositoryError>;
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait UserRepository: Debug + Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>, RepositoryError>;
    async fn find_random_valid_token(&self) -> Result<Option<String>, RepositoryError>;
    async fn find_token_by_user_id(
        &self,
        user_id: &Uuid,
    ) -> Result<Option<String>, RepositoryError>;
    async fn save(&self, user: &User) -> Result<(), RepositoryError>;
    async fn save_token(&self, user_id: &Uuid, access_token: &str) -> Result<(), RepositoryError>;
}
