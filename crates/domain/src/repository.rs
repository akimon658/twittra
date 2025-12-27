use std::sync::Arc;

use anyhow::Result;
use uuid::Uuid;

use crate::model::User;

#[derive(Clone)]
pub struct Repository {
    pub user: Arc<dyn UserRepository>,
}

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> Result<User>;
    async fn save(&self, user: &User) -> Result<()>;
    async fn save_token(&self, user_id: &Uuid, access_token: &str) -> Result<()>;
}
