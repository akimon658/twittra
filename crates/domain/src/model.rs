use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub handle: String,
}

#[derive(Serialize, ToSchema)]
pub struct UserToken {
    pub user_id: Uuid,
    pub access_token: String,
}
