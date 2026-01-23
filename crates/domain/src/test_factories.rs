#![cfg(any(test, feature = "test-utils"))]

use crate::model::{Message, MessageListItem, Reaction, Stamp, User};
use crate::repository::{
    MessageRepository, MockMessageRepository, MockStampRepository, MockUserRepository, Repository,
    StampRepository, UserRepository,
};
use fake::{Fake, Faker, faker::time::en::DateTimeBetween, uuid::UUIDv4};
use std::sync::Arc;
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

/// Generate a random timestamp close to the current time for testing.
///
/// This is useful for tests that need to simulate recent activity,
/// such as messages created within the last 24 hours.
pub fn fake_recent_datetime() -> OffsetDateTime {
    let now = OffsetDateTime::now_utc();
    let start = now - time::Duration::hours(23);
    let end = now;
    DateTimeBetween(start, end).fake()
}

pub struct MessageBuilder {
    id: Uuid,
    user_id: Uuid,
    channel_id: Uuid,
    content: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    reactions: Vec<Reaction>,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self {
            id: UUIDv4.fake(),
            user_id: UUIDv4.fake(),
            channel_id: UUIDv4.fake(),
            content: Faker.fake::<String>(),
            created_at: fake_datetime(),
            updated_at: fake_datetime(),
            reactions: vec![],
        }
    }

    pub fn id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn channel_id(mut self, channel_id: Uuid) -> Self {
        self.channel_id = channel_id;
        self
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    pub fn created_at(mut self, created_at: OffsetDateTime) -> Self {
        self.created_at = created_at;
        self
    }

    pub fn updated_at(mut self, updated_at: OffsetDateTime) -> Self {
        self.updated_at = updated_at;
        self
    }

    pub fn reactions(mut self, reactions: Vec<Reaction>) -> Self {
        self.reactions = reactions;
        self
    }

    pub fn build(self) -> Message {
        Message {
            id: self.id,
            user_id: self.user_id,
            channel_id: self.channel_id,
            content: self.content,
            created_at: self.created_at,
            updated_at: self.updated_at,
            reactions: self.reactions,
        }
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MessageListItemBuilder {
    id: Uuid,
    user_id: Uuid,
    user: Option<User>,
    channel_id: Uuid,
    content: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    reactions: Vec<Reaction>,
}

impl MessageListItemBuilder {
    pub fn new() -> Self {
        Self {
            id: UUIDv4.fake(),
            user_id: UUIDv4.fake(),
            user: None,
            channel_id: UUIDv4.fake(),
            content: Faker.fake::<String>(),
            created_at: fake_datetime(),
            updated_at: fake_datetime(),
            reactions: vec![],
        }
    }

    pub fn id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn user(mut self, user: Option<User>) -> Self {
        self.user = user;
        self
    }

    pub fn channel_id(mut self, channel_id: Uuid) -> Self {
        self.channel_id = channel_id;
        self
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    pub fn created_at(mut self, created_at: OffsetDateTime) -> Self {
        self.created_at = created_at;
        self
    }

    pub fn updated_at(mut self, updated_at: OffsetDateTime) -> Self {
        self.updated_at = updated_at;
        self
    }

    pub fn reactions(mut self, reactions: Vec<Reaction>) -> Self {
        self.reactions = reactions;
        self
    }

    pub fn build(self) -> MessageListItem {
        MessageListItem {
            id: self.id,
            user_id: self.user_id,
            user: self.user,
            channel_id: self.channel_id,
            content: self.content,
            created_at: self.created_at,
            updated_at: self.updated_at,
            reactions: self.reactions,
        }
    }
}

impl Default for MessageListItemBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct UserBuilder {
    id: Uuid,
    handle: String,
    display_name: String,
}

impl UserBuilder {
    pub fn new() -> Self {
        Self {
            id: UUIDv4.fake(),
            handle: Faker.fake::<String>(),
            display_name: Faker.fake::<String>(),
        }
    }

    pub fn id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn handle(mut self, handle: impl Into<String>) -> Self {
        self.handle = handle.into();
        self
    }

    pub fn display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = display_name.into();
        self
    }

    pub fn build(self) -> User {
        User {
            id: self.id,
            handle: self.handle,
            display_name: self.display_name,
        }
    }
}

impl Default for UserBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StampBuilder {
    id: Uuid,
    name: String,
}

impl StampBuilder {
    pub fn new() -> Self {
        Self {
            id: UUIDv4.fake(),
            name: Faker.fake::<String>(),
        }
    }

    pub fn id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn build(self) -> Stamp {
        Stamp {
            id: self.id,
            name: self.name,
        }
    }
}

impl Default for StampBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ReactionBuilder {
    stamp_id: Uuid,
    user_id: Uuid,
    stamp_count: i32,
}

impl ReactionBuilder {
    pub fn new() -> Self {
        Self {
            stamp_id: UUIDv4.fake(),
            user_id: UUIDv4.fake(),
            stamp_count: (1..100).fake(),
        }
    }

    pub fn stamp_id(mut self, stamp_id: Uuid) -> Self {
        self.stamp_id = stamp_id;
        self
    }

    pub fn user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn stamp_count(mut self, stamp_count: i32) -> Self {
        self.stamp_count = stamp_count;
        self
    }

    pub fn build(self) -> Reaction {
        Reaction {
            stamp_id: self.stamp_id,
            user_id: self.user_id,
            stamp_count: self.stamp_count,
        }
    }
}

impl Default for ReactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating Repository instances with mock repositories in tests.
///
/// This builder provides a fluent API for configuring Repository with custom
/// mock repositories. Any repository not explicitly set will use a default mock.
///
/// # Example
///
/// ```rust,ignore
/// let repo = RepositoryBuilder::new()
///     .message(mock_message_repo)
///     .user(mock_user_repo)
///     .build();
/// ```
pub struct RepositoryBuilder {
    message: Option<Arc<dyn MessageRepository>>,
    stamp: Option<Arc<dyn StampRepository>>,
    user: Option<Arc<dyn UserRepository>>,
}

impl RepositoryBuilder {
    /// Create a new builder with all repositories unset (will use defaults)
    pub fn new() -> Self {
        Self {
            message: None,
            stamp: None,
            user: None,
        }
    }

    /// Set a custom MessageRepository (default: MockMessageRepository::new())
    pub fn message<T: MessageRepository + 'static>(mut self, repo: T) -> Self {
        self.message = Some(Arc::new(repo));
        self
    }

    /// Set a custom StampRepository (default: MockStampRepository::new())
    pub fn stamp<T: StampRepository + 'static>(mut self, repo: T) -> Self {
        self.stamp = Some(Arc::new(repo));
        self
    }

    /// Set a custom UserRepository (default: MockUserRepository::new())
    pub fn user<T: UserRepository + 'static>(mut self, repo: T) -> Self {
        self.user = Some(Arc::new(repo));
        self
    }

    /// Build the Repository using provided repositories or default mocks.
    pub fn build(self) -> Repository {
        Repository {
            message: self
                .message
                .unwrap_or_else(|| Arc::new(MockMessageRepository::new())),
            stamp: self
                .stamp
                .unwrap_or_else(|| Arc::new(MockStampRepository::new())),
            user: self
                .user
                .unwrap_or_else(|| Arc::new(MockUserRepository::new())),
        }
    }
}

impl Default for RepositoryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
