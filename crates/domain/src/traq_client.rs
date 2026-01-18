use crate::error::TraqClientError;
use std::fmt::Debug;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::model::{Message, Stamp, User};

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait TraqClient: Debug + Send + Sync {
    async fn fetch_messages_since(
        &self,
        token: &str,
        since: OffsetDateTime,
    ) -> Result<Vec<Message>, TraqClientError>;
    async fn get_stamp(&self, token: &str, stamp_id: &Uuid) -> Result<Stamp, TraqClientError>;
    async fn get_stamps(&self, token: &str) -> Result<Vec<Stamp>, TraqClientError>;
    async fn get_stamp_image(
        &self,
        token: &str,
        stamp_id: &Uuid,
    ) -> Result<(Vec<u8>, String), TraqClientError>;
    async fn get_user(&self, token: &str, user_id: &Uuid) -> Result<User, TraqClientError>;

    async fn get_user_icon(
        &self,
        token: &str,
        user_id: &Uuid,
    ) -> Result<(Vec<u8>, String), TraqClientError>;
    async fn add_message_stamp(
        &self,
        token: &str,
        message_id: &Uuid,
        stamp_id: &Uuid,
        count: i32,
    ) -> Result<(), TraqClientError>;
    async fn remove_message_stamp(
        &self,
        token: &str,
        message_id: &Uuid,
        stamp_id: &Uuid,
    ) -> Result<(), TraqClientError>;
    async fn get_message(&self, token: &str, message_id: &Uuid)
    -> Result<Message, TraqClientError>;
}
