use anyhow::Result;
use domain::repository::Repository;
use sqlx::MySqlPool;
use std::sync::Arc;

use crate::repository::mariadb::{
    message::MariaDbMessageRepository, stamp::MariaDbStampRepository, user::MariaDbUserRepository,
};

pub mod message;
pub mod stamp;
pub mod user;

pub async fn new_repository(pool: MySqlPool) -> Result<Repository> {
    sqlx::migrate!().run(&pool).await?;

    Ok(Repository {
        message: Arc::new(MariaDbMessageRepository::new(pool.clone())),
        stamp: Arc::new(MariaDbStampRepository::new(pool.clone())),
        user: Arc::new(MariaDbUserRepository::new(pool)),
    })
}
