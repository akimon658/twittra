use anyhow::Result;
use sqlx::MySqlPool;

use crate::repository::mysql::user::MySqlUserRepository;

pub mod user;

pub struct MySqlRepository {
    pub user: MySqlUserRepository,
}

impl MySqlRepository {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = MySqlPool::connect(database_url).await?;

        sqlx::migrate!().run(&pool).await?;

        Ok(Self {
            user: MySqlUserRepository::new(pool),
        })
    }
}
