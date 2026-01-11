use serde::Serialize;
use time::{OffsetDateTime, error::Parse, format_description::well_known::Rfc3339};
use traq::models::{self, MessageStamp, MyUserDetail, UserDetail};
use utoipa::ToSchema;
use uuid::Uuid;

pub struct Message {
    pub id: Uuid,
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub content: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub reactions: Vec<Reaction>,
}

impl TryFrom<models::Message> for Message {
    type Error = Parse;

    fn try_from(value: models::Message) -> Result<Self, Self::Error> {
        Ok(Message {
            id: value.id,
            user_id: value.user_id,
            channel_id: value.channel_id,
            content: value.content,
            created_at: OffsetDateTime::parse(&value.created_at, &Rfc3339)?,
            updated_at: OffsetDateTime::parse(&value.updated_at, &Rfc3339)?,
            reactions: value.stamps.into_iter().map(Reaction::from).collect(),
        })
    }
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessageListItem {
    pub id: Uuid,
    pub user_id: Uuid,
    /// The user who posted the message.
    /// Omitted if the server hasn't cached the user info.
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    pub channel_id: Uuid,
    pub content: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub reactions: Vec<Reaction>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Reaction {
    pub stamp_id: Uuid,
    pub user_id: Uuid,
    pub stamp_count: i32,
}

impl From<MessageStamp> for Reaction {
    fn from(value: MessageStamp) -> Self {
        Reaction {
            stamp_id: value.stamp_id,
            user_id: value.user_id,
            stamp_count: value.count,
        }
    }
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub handle: String,
    pub display_name: String,
}

impl From<MyUserDetail> for User {
    fn from(value: MyUserDetail) -> Self {
        User {
            id: value.id,
            handle: value.name,
            display_name: value.display_name,
        }
    }
}

impl From<UserDetail> for User {
    fn from(value: UserDetail) -> Self {
        User {
            id: value.id,
            handle: value.name,
            display_name: value.display_name,
        }
    }
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserToken {
    pub user_id: Uuid,
    pub access_token: String,
}
