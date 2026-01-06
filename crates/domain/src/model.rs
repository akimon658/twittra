use serde::Serialize;
use time::{OffsetDateTime, error::Parse, format_description::well_known::Rfc3339};
use traq::models::{self, MyUserDetail, UserDetail};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: Uuid,
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub content: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
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
        })
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
