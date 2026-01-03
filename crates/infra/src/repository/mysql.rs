use std::sync::Arc;

use anyhow::Result;
use domain::repository::Repository;
use sqlx::MySqlPool;

use crate::repository::mysql::{message::MySqlMessageRepository, user::MySqlUserRepository};

pub mod message;
pub mod user;

pub async fn new_repository(pool: MySqlPool) -> Result<Repository> {
    sqlx::migrate!().run(&pool).await?;

    Ok(Repository {
        message: Arc::new(MySqlMessageRepository::new(pool.clone())),
        user: Arc::new(MySqlUserRepository::new(pool)),
    })
}
