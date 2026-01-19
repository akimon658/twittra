use domain::{error::RepositoryError, repository::Repository};
use sqlx::MySqlPool;
use std::sync::Arc;

use crate::repository::mariadb::{
    message::MariaDbMessageRepository, stamp::MariaDbStampRepository, user::MariaDbUserRepository,
};

pub mod message;
pub mod stamp;
pub mod user;

pub async fn new_repository(pool: MySqlPool) -> Result<Repository, RepositoryError> {
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

    Ok(Repository {
        message: Arc::new(MariaDbMessageRepository::new(pool.clone())),
        stamp: Arc::new(MariaDbStampRepository::new(pool.clone())),
        user: Arc::new(MariaDbUserRepository::new(pool)),
    })
}
