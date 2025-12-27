use std::sync::Arc;

use anyhow::Result;
use domain::repository::Repository;
use sqlx::MySqlPool;

use crate::repository::mysql::user::MySqlUserRepository;

pub mod user;

pub async fn new_repository(database_url: &str) -> Result<Repository> {
    let pool = MySqlPool::connect(database_url).await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(Repository {
        user: Arc::new(MySqlUserRepository::new(pool)),
    })
}
