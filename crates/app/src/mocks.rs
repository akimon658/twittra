use domain::{
    model::{Message, MessageListItem, Stamp, User},
    repository::{MessageRepository, StampRepository, UserRepository},
    traq_client::TraqClient,
};
use mockall::mock;
use std::fmt::Debug;
use time::OffsetDateTime;
use uuid::Uuid;

mock! {
    pub MessageRepository {}

    impl Debug for MessageRepository {
        fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> std::fmt::Result;
    }

    #[async_trait::async_trait]
    impl MessageRepository for MessageRepository {
        async fn find_latest_message_time(&self) -> anyhow::Result<Option<OffsetDateTime>>;
        async fn find_recent_messages(&self) -> anyhow::Result<Vec<MessageListItem>>;
        async fn remove_reaction(
            &self,
            message_id: &Uuid,
            stamp_id: &Uuid,
            user_id: &Uuid,
        ) -> anyhow::Result<()>;
        async fn save(&self, message: &Message) -> anyhow::Result<()>;
        async fn save_batch(&self, messages: &[Message]) -> anyhow::Result<()>;
    }
}

mock! {
    pub StampRepository {}

    impl Debug for StampRepository {
        fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> std::fmt::Result;
    }

    #[async_trait::async_trait]
    impl StampRepository for StampRepository {
        async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Stamp>>;
        async fn save(&self, stamp: &Stamp) -> anyhow::Result<()>;
        async fn save_batch(&self, stamps: &[Stamp]) -> anyhow::Result<()>;
    }
}

mock! {
    pub UserRepository {}

    impl Debug for UserRepository {
        fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> std::fmt::Result;
    }

    #[async_trait::async_trait]
    impl UserRepository for UserRepository {
        async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<User>>;
        async fn find_random_valid_token(&self) -> anyhow::Result<Option<String>>;
        async fn find_token_by_user_id(&self, user_id: &Uuid) -> anyhow::Result<Option<String>>;
        async fn save(&self, user: &User) -> anyhow::Result<()>;
        async fn save_token(&self, user_id: &Uuid, access_token: &str) -> anyhow::Result<()>;
    }
}

mock! {
    pub TraqClient {}

    impl Debug for TraqClient {
        fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> std::fmt::Result;
    }

    #[async_trait::async_trait]
    impl TraqClient for TraqClient {
        async fn fetch_messages_since(
            &self,
            token: &str,
            since: OffsetDateTime,
        ) -> anyhow::Result<Vec<Message>>;
        async fn get_stamp(&self, token: &str, stamp_id: &Uuid) -> anyhow::Result<Stamp>;
        async fn get_stamps(&self, token: &str) -> anyhow::Result<Vec<Stamp>>;
        async fn get_stamp_image(&self, token: &str, stamp_id: &Uuid) -> anyhow::Result<(Vec<u8>, String)>;
        async fn get_user(&self, token: &str, user_id: &Uuid) -> anyhow::Result<User>;
        async fn get_user_icon(&self, token: &str, user_id: &Uuid) -> anyhow::Result<(Vec<u8>, String)>;
        async fn add_message_stamp(
            &self,
            token: &str,
            message_id: &Uuid,
            stamp_id: &Uuid,
            count: i32,
        ) -> anyhow::Result<()>;
        async fn remove_message_stamp(
            &self,
            token: &str,
            message_id: &Uuid,
            stamp_id: &Uuid,
        ) -> anyhow::Result<()>;
        async fn get_message(&self, token: &str, message_id: &Uuid) -> anyhow::Result<Message>;
    }
}
