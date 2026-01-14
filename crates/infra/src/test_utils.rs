use anyhow::Result;
use sqlx::{MySql, MySqlPool, Pool};
use std::sync::Arc;
use testcontainers::{
    core::ContainerPort,
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};

/// Test infrastructure for managing Docker containers via Testcontainers.
/// Follows rucQ pattern: shared containers with per-test database isolation.
pub struct TestInfra {
    /// MariaDB container (shared across tests)
    _mariadb: ContainerAsync<GenericImage>,
    /// Base database pool (for creating test databases)
    admin_pool: Pool<MySql>,
    /// MariaDB connection host
    db_host: String,
    /// MariaDB connection port
    db_port: u16,
}

impl TestInfra {
    /// Initialize test infrastructure.
    /// Starts MariaDB container and creates admin connection pool.
    pub async fn new() -> Result<Self> {
        // Start MariaDB container
        let mariadb_image = GenericImage::new("mariadb", "10.11.15-jammy")
            .with_exposed_port(ContainerPort::Tcp(3306))
            .with_env_var("MARIADB_ROOT_PASSWORD", "password");
        
        let mariadb: ContainerAsync<GenericImage> = mariadb_image.start().await?;
        
        let db_host: String = mariadb.get_host().await?.to_string();
        let db_port = mariadb.get_host_port_ipv4(3306).await?;

        // Wait a bit for MariaDB to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // Create admin pool for database management
        let connection_string = format!(
            "mysql://root:password@{}:{}/mysql",
            db_host, db_port
        );
        
        let admin_pool = MySqlPool::connect(&connection_string).await?;

        Ok(Self {
            _mariadb: mariadb,
            admin_pool,
            db_host,
            db_port,
        })
    }

    /// Create a new test database with a unique name.
    /// Returns a connection pool to the new database.
    pub async fn create_test_database(&self, test_name: &str) -> Result<Pool<MySql>> {
        // Generate unique database name
        let db_name = format!("test_{}_{}", test_name, fastrand::u64(..));

        // Create database
        sqlx::query(&format!("CREATE DATABASE `{}`", db_name))
            .execute(&self.admin_pool)
            .await?;

        // Connect to new database
        let connection_string = format!(
            "mysql://root:password@{}:{}/{}",
            self.db_host, self.db_port, db_name
        );

        let pool = MySqlPool::connect(&connection_string).await?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;

        Ok(pool)
    }
}

/// Global test infrastructure instance.
static TEST_INFRA: tokio::sync::OnceCell<Arc<TestInfra>> = tokio::sync::OnceCell::const_new();

/// Get or initialize the global test infrastructure.
pub async fn get_test_infra() -> Result<Arc<TestInfra>> {
    TEST_INFRA
        .get_or_try_init(|| async {
            let infra = TestInfra::new().await?;
            Ok::<Arc<TestInfra>, anyhow::Error>(Arc::new(infra))
        })
        .await
        .map(Arc::clone)
        .map_err(|e| anyhow::anyhow!(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_infra_setup() {
        let infra = get_test_infra().await.unwrap();
        let pool = infra.create_test_database("setup_test").await.unwrap();
        
        // Verify we can query the database
        let result: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        
        assert_eq!(result.0, 1);
    }
}
