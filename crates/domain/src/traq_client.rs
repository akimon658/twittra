use anyhow::Result;
use time::PrimitiveDateTime;

use crate::model::Message;

#[async_trait::async_trait]
pub trait TraqClient: Send + Sync {
    async fn fetch_messages_since(
        &self,
        token: &str,
        after: PrimitiveDateTime,
    ) -> Result<Vec<Message>>;
}
