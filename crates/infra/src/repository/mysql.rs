use std::sync::Arc;

use anyhow::Result;
use domain::repository::Repository;
use sqlx::MySqlPool;

use crate::repository::mysql::user::MySqlUserRepository;

pub mod user;

pub async fn new_repository(pool: MySqlPool) -> Result<Repository> {
    sqlx::migrate!().run(&pool).await?;

    Ok(Repository {
        user: Arc::new(MySqlUserRepository::new(pool)),
    })
}
