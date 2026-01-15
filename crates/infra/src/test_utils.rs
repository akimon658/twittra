use anyhow::Result;
use sqlx::{Connection, MySql, Pool};
use std::sync::Arc;
use testcontainers::compose::DockerCompose;
use testcontainers::core::IntoContainerPort;

/// Test infrastructure for managing Docker containers via Docker Compose.
/// Follows rucQ pattern: shared containers with per-test database isolation.
pub struct TestInfra {
    /// Docker Compose instance (contains db, traq_server, etc.)
    _compose: DockerCompose,
    /// DB host
    db_host: String,
    /// DB port
    db_port: u16,
    /// Admin connection string for CREATE DATABASE operations
    admin_connection_string: String,
}

impl TestInfra {
    /// Initialize test infrastructure using Docker Compose.
    /// Starts services defined in compose.yaml.
    pub async fn new() -> Result<Self> {
        // Get path to compose.yaml in project root
        let compose_file_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("compose.yaml");

        // Start Docker Compose with local client
        // Set db port to 0 for random OS-assigned port
        // Don't set profile - without a profile, only services without profiles start (db only)
        let mut compose = DockerCompose::with_local_client(&[compose_file_path.as_path()])
            .with_env("MARIADB_PORT", "0");

        // Start services
        compose.up().await?;

        // Get MariaDB service
        let db_service = compose
            .service("db")
            .ok_or_else(|| anyhow::anyhow!("db service not found"))?;

        // Get host port for MariaDB
        let db_port = db_service.get_host_port_ipv4(3306.tcp()).await?;
        let db_host = "127.0.0.1".to_string();

        // Wait for MariaDB to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        // Create admin connection string (not a pool!)
        // Avoiding shared pool to prevent deadlock described in sqlx #3953
        let admin_connection_string = format!(
            "mysql://root:password@{}:{}/mysql",
            db_host, db_port
        );

        Ok(Self {
            _compose: compose,
            db_host,
            db_port,
            admin_connection_string,
        })
    }

    /// Create a new test database with a unique name.
    /// Returns a connection pool to the new database.
    pub async fn create_test_database(&self, test_name: &str) -> Result<Pool<MySql>> {
        // Generate unique database name
        let db_name = format!("test_{}_{}", test_name, fastrand::u64(..));
        
        println!("[DEBUG] Creating test database: {}", db_name);

        // Create a SINGLE-USE connection specifically for CREATE DATABASE
        // This avoids the connection pool deadlock issue described in sqlx #3953
        // https://github.com/launchbadge/sqlx/issues/3953
        let mut admin_conn = sqlx::MySqlConnection::connect(&self.admin_connection_string).await?;
        
        println!("[DEBUG] Connected to mysql database for CREATE");

        // Create database using the single-use connection
        sqlx::query(&format!("CREATE DATABASE `{}`", db_name))
            .execute(&mut admin_conn)
            .await?;
        
        // Explicitly close the admin connection
        admin_conn.close().await?;
        
        println!("[DEBUG] Database {} created, admin connection closed", db_name);

        // Connect to new database with explicit pool configuration
        let connection_string = format!(
            "mysql://root:password@{}:{}/{}",
            self.db_host, self.db_port, db_name
        );

        println!("[DEBUG] Connecting to {} with max_connections=10", db_name);
        
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(10) // Increased: migrations may need multiple connections
            .min_connections(0) // Lazy initialization to avoid connection exhaustion
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(&connection_string)
            .await?;
        
        println!("[DEBUG] Pool connection ESTABLISHED for {}", db_name);
        println!("[DEBUG] Pool created for {}, running migrations...", db_name);

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;
        
        println!("[DEBUG] Migrations complete for {}", db_name);

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
