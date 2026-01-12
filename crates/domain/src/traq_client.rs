use anyhow::Result;
use std::fmt::Debug;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::model::{Message, Stamp, User};

#[async_trait::async_trait]
pub trait TraqClient: Debug + Send + Sync {
    async fn fetch_messages_since(
        &self,
        token: &str,
        since: OffsetDateTime,
    ) -> Result<Vec<Message>>;
    async fn get_stamp(&self, token: &str, stamp_id: &Uuid) -> Result<Stamp>;
    async fn get_user(&self, token: &str, user_id: &Uuid) -> Result<User>;

    async fn get_user_icon(&self, token: &str, user_id: &Uuid) -> Result<(Vec<u8>, String)>;
}
