use domain::{
    error::TraqClientError,
    model::{Message, Stamp, User},
    traq_client::TraqClient,
};
use time::{OffsetDateTime, error::Parse, format_description::well_known::Rfc3339};
use traq::{
    apis::{configuration::Configuration, message_api, stamp_api, user_api},
    models::PostMessageStampRequest,
};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct TraqClientImpl {
    base_url: String,
}

impl TraqClientImpl {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

#[async_trait::async_trait]
impl TraqClient for TraqClientImpl {
    async fn fetch_messages_since(
        &self,
        token: &str,
        since: OffsetDateTime,
    ) -> Result<Vec<Message>, TraqClientError> {
        let config = Configuration {
            base_path: self.base_url.clone(),
            oauth_access_token: Some(token.to_string()),
            ..Default::default()
        };
        let search_result = message_api::search_messages(
            &config,
            None,
            Some(
                since
                    .format(&Rfc3339)
                    .map_err(|e| TraqClientError::ResponseParse(e.to_string()))?,
            ),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .map_err(|e| TraqClientError::ApiError(e.to_string()))?;
        let messages = search_result
            .hits
            .into_iter()
            .map(|msg| msg.try_into())
            .collect::<Result<Vec<Message>, _>>()
            .map_err(|e: Parse| TraqClientError::ResponseParse(e.to_string()))?;

        Ok(messages)
    }

    async fn get_stamp(&self, token: &str, stamp_id: &Uuid) -> Result<Stamp, TraqClientError> {
        let config = Configuration {
            base_path: self.base_url.clone(),
            oauth_access_token: Some(token.to_string()),
            ..Default::default()
        };
        let traq_stamp = stamp_api::get_stamp(&config, &stamp_id.to_string())
            .await
            .map_err(|e| TraqClientError::ApiError(e.to_string()))?;
        let stamp = traq_stamp.into();

        Ok(stamp)
    }

    async fn get_stamps(&self, token: &str) -> Result<Vec<Stamp>, TraqClientError> {
        let config = Configuration {
            base_path: self.base_url.clone(),
            oauth_access_token: Some(token.to_string()),
            ..Default::default()
        };
        let traq_stamps = stamp_api::get_stamps(&config, None, None)
            .await
            .map_err(|e| TraqClientError::ApiError(e.to_string()))?;
        let stamps = traq_stamps.into_iter().map(|s| s.into()).collect();

        Ok(stamps)
    }

    async fn get_stamp_image(
        &self,
        token: &str,
        stamp_id: &Uuid,
    ) -> Result<(Vec<u8>, String), TraqClientError> {
        let config = Configuration {
            base_path: self.base_url.clone(),
            oauth_access_token: Some(token.to_string()),
            ..Default::default()
        };
        let response = stamp_api::get_stamp_image(&config, &stamp_id.to_string())
            .await
            .map_err(|e| TraqClientError::ApiError(e.to_string()))?;
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        let bytes = response
            .bytes()
            .await
            .map_err(|e| TraqClientError::HttpRequest(e.to_string()))?
            .to_vec();
        Ok((bytes, content_type))
    }

    async fn get_user(&self, token: &str, user_id: &Uuid) -> Result<User, TraqClientError> {
        let config = Configuration {
            base_path: self.base_url.clone(),
            oauth_access_token: Some(token.to_string()),
            ..Default::default()
        };
        let traq_user = user_api::get_user(&config, &user_id.to_string())
            .await
            .map_err(|e| TraqClientError::ApiError(e.to_string()))?;
        let user = traq_user.into();

        Ok(user)
    }

    async fn get_user_icon(
        &self,
        token: &str,
        user_id: &Uuid,
    ) -> Result<(Vec<u8>, String), TraqClientError> {
        let config = Configuration {
            base_path: self.base_url.clone(),
            oauth_access_token: Some(token.to_string()),
            ..Default::default()
        };
        let response = user_api::get_user_icon(&config, &user_id.to_string())
            .await
            .map_err(|e| TraqClientError::ApiError(e.to_string()))?;
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        let bytes = response
            .bytes()
            .await
            .map_err(|e| TraqClientError::HttpRequest(e.to_string()))?
            .to_vec();
        Ok((bytes, content_type))
    }

    async fn add_message_stamp(
        &self,
        token: &str,
        message_id: &Uuid,
        stamp_id: &Uuid,
        count: i32,
    ) -> Result<(), TraqClientError> {
        let config = Configuration {
            base_path: self.base_url.clone(),
            oauth_access_token: Some(token.to_string()),
            ..Default::default()
        };
        let post_message_stamp_request = PostMessageStampRequest { count };
        message_api::add_message_stamp(
            &config,
            &message_id.to_string(),
            &stamp_id.to_string(),
            Some(post_message_stamp_request),
        )
        .await
        .map_err(|e| TraqClientError::ApiError(e.to_string()))?;

        Ok(())
    }

    async fn remove_message_stamp(
        &self,
        token: &str,
        message_id: &Uuid,
        stamp_id: &Uuid,
    ) -> Result<(), TraqClientError> {
        let config = Configuration {
            base_path: self.base_url.clone(),
            oauth_access_token: Some(token.to_string()),
            ..Default::default()
        };
        message_api::remove_message_stamp(&config, &message_id.to_string(), &stamp_id.to_string())
            .await
            .map_err(|e| TraqClientError::ApiError(e.to_string()))?;

        Ok(())
    }

    async fn get_message(
        &self,
        token: &str,
        message_id: &Uuid,
    ) -> Result<Message, TraqClientError> {
        let config = Configuration {
            base_path: self.base_url.clone(),
            oauth_access_token: Some(token.to_string()),
            ..Default::default()
        };
        let message = message_api::get_message(&config, &message_id.to_string())
            .await
            .map_err(|e| TraqClientError::ApiError(e.to_string()))?;
        let message = message
            .try_into()
            .map_err(|e: Parse| TraqClientError::ResponseParse(e.to_string()))?;

        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::time::Duration;
    use oauth2::{
        AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, Scope, TokenResponse,
        TokenUrl, basic::BasicClient,
    };
    use reqwest::{StatusCode, redirect::Policy};
    use std::path::PathBuf;
    use testcontainers::{compose::DockerCompose, core::wait::HttpWaitStrategy};
    use uuid::Uuid;

    /// Test environment that orchestrates traQ via Docker Compose
    struct TraqTestEnvironment {
        compose: Option<DockerCompose>,
        base_url: String,
        admin_token: Option<String>,
        admin_user_id: Option<Uuid>,
    }

    impl TraqTestEnvironment {
        async fn start() -> Self {
            // Get workspace root (crates/infra/src -> project root)
            let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .to_path_buf();

            let compose_file = workspace_root.join("compose.yaml");
            let test_compose_file = workspace_root.join("compose.test.yaml");

            // Create and configure compose instance
            let mut compose = DockerCompose::with_local_client(&[
                compose_file.to_str().unwrap(),
                test_compose_file.to_str().unwrap(),
            ])
            .with_env("TRAQ_SERVER_PORT", "0") // Random port assignment
            .with_env("MARIADB_PORT", "0")
            .with_wait_for_service(
                "traq_server",
                HttpWaitStrategy::new("/api/v3/version")
                    .with_expected_status_code(StatusCode::OK)
                    .into(),
            );

            // Start services and wait for readiness
            compose.up().await.expect("Failed to start docker compose");

            // Get traq_server service and mapped port
            let traq_server = compose
                .service("traq_server")
                .expect("traq_server service not found");
            let port = traq_server
                .get_host_port_ipv4(3000)
                .await
                .expect("Failed to get port");

            // base_url for traQ API (with /api/v3 prefix)
            let api_base_url = format!("http://localhost:{}/api/v3", port);
            // server_base_url for OAuth endpoints (no /api/v3)
            let server_base_url = format!("http://localhost:{}", port);

            // Initialize traQ and get admin token via OAuth2
            let (admin_token, admin_user_id) =
                match Self::initialize_traq_oauth(&api_base_url, &server_base_url).await {
                    Ok(result) => (Some(result.0), Some(result.1)),
                    Err(e) => {
                        eprintln!("WARNING: traQ initialization failed: {}", e);
                        (None, None)
                    }
                };

            Self {
                compose: Some(compose),
                base_url: api_base_url,
                admin_token,
                admin_user_id,
            }
        }

        async fn initialize_traq_oauth(
            api_base_url: &str,
            _server_base_url: &str,
        ) -> Result<(String, Uuid), String> {
            let client = reqwest::Client::builder()
                .cookie_store(true)
                .redirect(Policy::none())
                .build()
                .map_err(|e| e.to_string())?;

            // Login with default user (traq/traq)
            eprintln!("Logging in with default user (traq/traq)...");
            let login_res = client
                .post(format!("{}/login", api_base_url))
                .json(&serde_json::json!({
                    "name": "traq",
                    "password": "traq"
                }))
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !login_res.status().is_success() {
                return Err("Login failed with default user".to_string());
            }

            // Get user ID
            let me_res = client
                .get(format!("{}/users/me", api_base_url))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let me_data: serde_json::Value = me_res.json().await.map_err(|e| e.to_string())?;
            let user_id =
                Uuid::parse_str(me_data["id"].as_str().unwrap()).map_err(|e| e.to_string())?;

            // Create OAuth client
            eprintln!("Creating OAuth client...");
            let client_res = client
                .post(format!("{}/clients", api_base_url))
                .json(&serde_json::json!({
                    "name": "test_client",
                    "description": "Test client for integration testing",
                    "callbackUrl": "http://localhost:3000/callback",
                    "scopes": ["read", "write"]
                }))
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !client_res.status().is_success() {
                let error = client_res.text().await.map_err(|e| e.to_string())?;
                return Err(format!("Failed to create OAuth client: {}", error));
            }

            let client_data: serde_json::Value =
                client_res.json().await.map_err(|e| e.to_string())?;
            let client_id = client_data["id"].as_str().unwrap();
            let client_secret = client_data["secret"].as_str().unwrap();

            // Set up OAuth2 client
            let oauth_client = BasicClient::new(ClientId::new(client_id.to_string()))
                .set_client_secret(ClientSecret::new(client_secret.to_string()))
                .set_auth_uri(
                    AuthUrl::new(format!("{}/oauth2/authorize", api_base_url))
                        .map_err(|e| e.to_string())?,
                )
                .set_token_uri(
                    TokenUrl::new(format!("{}/oauth2/token", api_base_url))
                        .map_err(|e| e.to_string())?,
                );

            // Generate authorization URL
            let (auth_url, _csrf_state) = oauth_client
                .authorize_url(CsrfToken::new_random)
                .add_scope(Scope::new("read".to_string()))
                .add_scope(Scope::new("write".to_string()))
                .url();

            // Get authorization (redirects to consent)
            let auth_res = client
                .get(auth_url.as_str())
                .send()
                .await
                .map_err(|e| e.to_string())?;

            // Extract authorization code
            let code = if auth_res.status().is_redirection() {
                let location = auth_res
                    .headers()
                    .get("location")
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| "No location header".to_string())?;

                if location.contains("/consent") {
                    // Approve via /oauth2/authorize/decide
                    let decide_url = format!("{}/oauth2/authorize/decide", api_base_url);
                    let approve_res = client
                        .post(&decide_url)
                        .form(&[("submit", "approve")])
                        .send()
                        .await
                        .map_err(|e| e.to_string())?;

                    if !approve_res.status().is_redirection() {
                        return Err("Consent approval failed".to_string());
                    }

                    let location = approve_res
                        .headers()
                        .get("location")
                        .and_then(|v| v.to_str().ok())
                        .ok_or_else(|| "No location after approval".to_string())?;

                    let callback_url = url::Url::parse(location).map_err(|e| e.to_string())?;
                    callback_url
                        .query_pairs()
                        .find(|(key, _)| key == "code")
                        .map(|(_, value)| value.to_string())
                        .ok_or_else(|| "No code in callback".to_string())?
                } else {
                    let callback_url = url::Url::parse(location).map_err(|e| e.to_string())?;
                    callback_url
                        .query_pairs()
                        .find(|(key, _)| key == "code")
                        .map(|(_, value)| value.to_string())
                        .ok_or_else(|| "No code in callback".to_string())?
                }
            } else {
                return Err("Unexpected authorization response".to_string());
            };

            // Exchange code for token
            let http_client = reqwest::Client::new();
            let token_result = oauth_client
                .exchange_code(AuthorizationCode::new(code))
                .request_async(&http_client)
                .await
                .map_err(|e| e.to_string())?;

            let access_token = token_result.access_token().secret().to_string();
            eprintln!("Got access token!");

            Ok((access_token, user_id))
        }

        fn base_url(&self) -> &str {
            &self.base_url
        }

        fn admin_token(&self) -> &str {
            self.admin_token.as_deref().unwrap_or("")
        }

        fn admin_user_id(&self) -> Option<Uuid> {
            self.admin_user_id
        }

        /// Explicit cleanup method
        async fn cleanup(mut self) {
            if let Some(compose) = self.compose.take() {
                eprintln!("Cleaning up Docker Compose resources...");
                compose
                    .down()
                    .await
                    .expect("Failed to cleanup docker compose");
                eprintln!("Cleanup complete!");
            }
        }
    }

    #[tokio::test]
    async fn test_environment_starts() {
        let env = TraqTestEnvironment::start().await;

        assert!(!env.base_url().is_empty());
        assert!(!env.admin_token().is_empty());
        assert!(env.admin_user_id().is_some());

        println!("traQ URL: {}", env.base_url());
        println!("Admin token: {}", env.admin_token());
        println!("Admin user ID: {:?}", env.admin_user_id());

        env.cleanup().await;
    }

    #[tokio::test]
    async fn test_get_user_success() {
        let env = TraqTestEnvironment::start().await;

        let client = TraqClientImpl::new(env.base_url().to_string());
        let user_id = env.admin_user_id().expect("No admin user ID");

        let result = client.get_user(env.admin_token(), &user_id).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, user_id);
        assert_eq!(user.handle, "traq");

        env.cleanup().await;
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let env = TraqTestEnvironment::start().await;

        let client = TraqClientImpl::new(env.base_url().to_string());
        let non_existent_id = Uuid::new_v4();

        let result = client.get_user(env.admin_token(), &non_existent_id).await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("404") || error_msg.contains("Not Found"));

        env.cleanup().await;
    }

    #[tokio::test]
    async fn test_get_user_unauthorized() {
        let env = TraqTestEnvironment::start().await;

        let client = TraqClientImpl::new(env.base_url().to_string());
        let user_id = env.admin_user_id().expect("No admin user ID");

        let result = client.get_user("invalid_token", &user_id).await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("401") || error_msg.contains("Unauthorized"));

        env.cleanup().await;
    }

    #[tokio::test]
    async fn test_get_stamps_success() {
        let env = TraqTestEnvironment::start().await;

        let client = TraqClientImpl::new(env.base_url().to_string());

        let result = client.get_stamps(env.admin_token()).await;

        assert!(result.is_ok());
        let stamps = result.unwrap();
        // traQ has default stamps
        assert!(!stamps.is_empty());

        env.cleanup().await;
    }

    #[tokio::test]
    async fn test_get_stamp_success() {
        let env = TraqTestEnvironment::start().await;

        let client = TraqClientImpl::new(env.base_url().to_string());

        // First get all stamps to get a valid ID
        let stamps = client
            .get_stamps(env.admin_token())
            .await
            .expect("Failed to get stamps");
        assert!(!stamps.is_empty());

        let stamp_id = stamps[0].id;

        // Now get individual stamp
        let result = client.get_stamp(env.admin_token(), &stamp_id).await;

        assert!(result.is_ok());
        let stamp = result.unwrap();
        assert_eq!(stamp.id, stamp_id);

        env.cleanup().await;
    }

    #[tokio::test]
    async fn test_fetch_messages_since() {
        let env = TraqTestEnvironment::start().await;

        let client = TraqClientImpl::new(env.base_url().to_string());

        // Search messages from a week ago
        let since = OffsetDateTime::now_utc() - Duration::days(7);

        let result = client.fetch_messages_since(env.admin_token(), since).await;

        assert!(result.is_ok());
        let messages = result.unwrap();
        // May be empty in freshly created traQ instance
        println!("Found {} messages", messages.len());

        env.cleanup().await;
    }
}
