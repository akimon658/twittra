use anyhow::Result;
use domain::{model::User, repository::UserRepository};
use sqlx::MySqlPool;
use uuid::Uuid;

pub struct MySqlUserRepository {
    pool: MySqlPool,
}

impl MySqlUserRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for MySqlUserRepository {
    async fn get_user(&self, id: &Uuid) -> Result<User> {
        unimplemented!()
    }

    async fn save_token(&self, user_id: &Uuid, access_token: &str) -> Result<()> {
        unimplemented!()
    }

    async fn save_user(&self, user: &User) -> Result<()> {
        unimplemented!()
    }
}
