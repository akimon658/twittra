use anyhow::Result;
use uuid::Uuid;

use crate::model::User;

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_user(&self, id: &Uuid) -> Result<User>;
    async fn save_token(&self, user_id: &Uuid, access_token: &str) -> Result<()>;
    async fn save_user(&self, user: &User) -> Result<()>;
}
