#![cfg(test)]

use domain::model::{Message, MessageListItem, Reaction, Stamp, User};
use fake::{Fake, Faker};
use time::OffsetDateTime;
use uuid::Uuid;

pub fn create_user() -> User {
    User {
        id: Uuid::now_v7(),
        handle: Faker.fake::<String>(),
        display_name: Faker.fake::<String>(),
    }
}

pub fn create_stamp() -> Stamp {
    Stamp {
        id: Uuid::now_v7(),
        name: Faker.fake::<String>(),
    }
}

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

pub fn create_reaction(stamp_id: Uuid, user_id: Uuid) -> Reaction {
    Reaction {
        stamp_id,
        user_id,
        stamp_count: (1..100).fake(),
    }
}

pub fn create_message_list_item() -> MessageListItem {
    MessageListItem {
        id: Uuid::now_v7(),
        user_id: Uuid::now_v7(),
        user: Some(create_user()),
        channel_id: Uuid::now_v7(),
        content: Faker.fake::<String>(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        reactions: vec![],
    }
}
