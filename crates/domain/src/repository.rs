use std::{fmt::Debug, sync::Arc};

use anyhow::Result;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::model::{Message, MessageListItem, Stamp, User};

#[derive(Clone, Debug)]
pub struct Repository {
    pub message: Arc<dyn MessageRepository>,
    pub stamp: Arc<dyn StampRepository>,
    pub user: Arc<dyn UserRepository>,
}

#[async_trait::async_trait]
pub trait MessageRepository: Debug + Send + Sync {
    async fn find_latest_message_time(&self) -> Result<Option<OffsetDateTime>>;
    async fn find_recent_messages(&self) -> Result<Vec<MessageListItem>>;
    /// Removes a reaction from a message.
    /// This is used for optimistic updates when deleting a stamp.
    async fn remove_reaction(
        &self,
        message_id: &Uuid,
        stamp_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<()>;
    /// Saves a batch of messages to the repository.
    /// It does nothing if `messages` is empty.
    async fn save_batch(&self, messages: &[Message]) -> Result<()>;
}

#[async_trait::async_trait]
pub trait StampRepository: Debug + Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Stamp>>;
    async fn save(&self, stamp: &Stamp) -> Result<()>;
}

#[async_trait::async_trait]
pub trait UserRepository: Debug + Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>>;
    async fn find_random_valid_token(&self) -> Result<Option<String>>;
    async fn find_token_by_user_id(&self, user_id: &Uuid) -> Result<Option<String>>;
    async fn save(&self, user: &User) -> Result<()>;
    async fn save_token(&self, user_id: &Uuid, access_token: &str) -> Result<()>;
}
