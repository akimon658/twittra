use serde::Serialize;
use time::{PrimitiveDateTime, error::Parse, format_description::BorrowedFormatItem, macros};
use traq::models::{self, MyUserDetail};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: Uuid,
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub content: String,
    pub created_at: PrimitiveDateTime,
    pub updated_at: PrimitiveDateTime,
}

const TIME_FORMAT: &[BorrowedFormatItem] =
    macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]Z");

impl TryFrom<models::Message> for Message {
    type Error = Parse;

    fn try_from(value: models::Message) -> Result<Self, Self::Error> {
        Ok(Message {
            id: value.id,
            user_id: value.user_id,
            channel_id: value.channel_id,
            content: value.content,
            created_at: PrimitiveDateTime::parse(&value.created_at, &TIME_FORMAT)?,
            updated_at: PrimitiveDateTime::parse(&value.updated_at, &TIME_FORMAT)?,
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

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserToken {
    pub user_id: Uuid,
    pub access_token: String,
}
