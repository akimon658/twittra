#![cfg(test)]

use domain::model::{Message, Reaction, Stamp, User};
use fake::{Fake, Faker};
use time::OffsetDateTime;
use uuid::Uuid;

/// Create a random user for testing
pub fn create_user() -> User {
    User {
        id: Uuid::now_v7(),
        handle: Faker.fake::<String>(),
        display_name: Faker.fake::<String>(),
    }
}

/// Create a random stamp for testing
pub fn create_stamp() -> Stamp {
    Stamp {
        id: Uuid::now_v7(),
        name: Faker.fake::<String>(),
    }
}

/// Create a random message for testing
pub fn create_message() -> Message {
    Message {
        id: Uuid::now_v7(),
        user_id: Uuid::now_v7(),
        channel_id: Uuid::now_v7(),
        content: Faker.fake::<String>(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        reactions: vec![],
    }
}

/// Create a random reaction for testing
pub fn create_reaction(stamp_id: Uuid, user_id: Uuid) -> Reaction {
    Reaction {
        stamp_id,
        user_id,
        stamp_count: (1..100).fake(),
    }
}

// Helper trait to allow overriding specific fields
pub trait WithId {
    fn with_id(self, id: Uuid) -> Self;
}

impl WithId for User {
    fn with_id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }
}

impl WithId for Message {
    fn with_id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }
}

impl WithId for Stamp {
    fn with_id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }
}
