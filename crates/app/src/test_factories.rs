#![cfg(test)]

use domain::model::{Message, MessageListItem, Reaction, Stamp, User};
use fake::{Fake, Faker, faker::time::en::DateTimeBetween, uuid::UUIDv4};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

/// Generate a random but valid timestamp for testing.
///
/// `DateTime().fake()` generates dates from approximately -9978 to 10000 AD,
/// but RFC3339 only supports dates between 0000 AD and 9999 AD.
/// Using `DateTimeBetween` ensures all generated timestamps are RFC3339-serializable.
fn fake_datetime() -> OffsetDateTime {
    let start = OffsetDateTime::parse("2020-01-01T00:00:00Z", &Rfc3339).unwrap();
    let end = OffsetDateTime::parse("2030-01-01T00:00:00Z", &Rfc3339).unwrap();
    DateTimeBetween(start, end).fake()
}

pub fn create_user() -> User {
    User {
        id: UUIDv4.fake(),
        handle: Faker.fake::<String>(),
        display_name: Faker.fake::<String>(),
    }
}

pub fn create_stamp() -> Stamp {
    Stamp {
        id: UUIDv4.fake(),
        name: Faker.fake::<String>(),
    }
}

pub fn create_message() -> Message {
    Message {
        id: UUIDv4.fake(),
        user_id: UUIDv4.fake(),
        channel_id: UUIDv4.fake(),
        content: Faker.fake::<String>(),
        created_at: fake_datetime(),
        updated_at: fake_datetime(),
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
        id: UUIDv4.fake(),
        user_id: UUIDv4.fake(),
        user: Some(create_user()),
        channel_id: UUIDv4.fake(),
        content: Faker.fake::<String>(),
        created_at: fake_datetime(),
        updated_at: fake_datetime(),
        reactions: vec![],
    }
}
