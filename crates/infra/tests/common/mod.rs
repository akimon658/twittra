use std::time::Duration;
use oauth2::{
    AuthUrl, ClientId, ClientSecret, TokenUrl,
    AuthorizationCode, CsrfToken, Scope,
    basic::BasicClient,
    TokenResponse,
};
use testcontainers::compose::DockerCompose;
use uuid::Uuid;

pub struct TraqTestEnvironment {
    _compose: DockerCompose,
    base_url: String,
    admin_token: Option<String>,
}

impl TraqTestEnvironment {
    pub async fn start() -> anyhow::Result<Self> {
        // Get workspace root (crates/infra/tests -> project root)
        let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        
        let compose_file = workspace_root.join("compose.yaml");
        
        // Generate unique project name
        let project_name = format!("traq_test_{}", Uuid::now_v7().simple());
        
        // Create and configure compose instance (builder pattern - with_env consumes self)
        let mut compose = DockerCompose::with_local_client(&[compose_file.to_str().unwrap()])
            .with_env("COMPOSE_PROJECT_NAME", &project_name)
            .with_env("COMPOSE_PROFILES", "dev")
            .with_env("TRAQ_SERVER_PORT", "0")  // Random port assignment
            .with_env("MARIADB_PORT", "0")
            .with_env("ADMINER_PORT", "0");
        
        // Start services
        compose.up().await?;
        
        // Wait for services to initialize
        tokio::time::sleep(Duration::from_secs(10)).await;
        
        // Get traq_server service and mapped port (not Caddy)
        let traq_server = compose.service("traq_server").expect("traq_server service not found");
        let port = traq_server.get_host_port_ipv4(3000).await?;
        
        // base_url for traQ API (with /api/v3 prefix)
        let api_base_url = format!("http://localhost:{}/api/v3", port);
        // server_base_url for OAuth endpoints (no /api/v3)
        let server_base_url = format!("http://localhost:{}", port);
        
        // Initialize traQ and get admin token via OAuth2
        let admin_token = Self::initialize_traq_oauth(&api_base_url, &server_base_url).await.ok();
        
        Ok(Self {
            _compose: compose,
            base_url: api_base_url,
            admin_token,
        })
    }
    
    async fn initialize_traq_oauth(api_base_url: &str, _server_base_url: &str) -> anyhow::Result<String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::none())  // Handle redirects manually
            .build()?;
        
        // Wait for traQ to be ready
        eprintln!("Waiting for traQ at {}...", api_base_url);
        for i in 0..60 {
            match client.get(format!("{}/version", api_base_url)).send().await {
                Ok(res) if res.status().is_success() => {
                    eprintln!("traQ ready!");
                    break;
                }
                Ok(res) => eprintln!("Status: {}", res.status()),
                Err(e) => eprintln!("Attempt {}/60: {}", i + 1, e),
            }
            if i == 59 {
                anyhow::bail!("traQ not ready after 60 attempts");
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        
        // Login with default user (traq/traq) to create OAuth client
        eprintln!("Logging in with default user (traq/traq)...");
        let login_res = client
            .post(format!("{}/login", api_base_url))
            .json(&serde_json::json!({
                "name": "traq",
                "password": "traq"
            }))
            .send()
            .await?;
        
        if !login_res.status().is_success() {
            anyhow::bail!("Login failed with default user");
        }
        
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
            .await?;
        
        if !client_res.status().is_success() {
            let error = client_res.text().await?;
            anyhow::bail!("Failed to create OAuth client: {}", error);
        }
        
        let client_data: serde_json::Value = client_res.json().await?;
        let client_id = client_data["id"].as_str().unwrap();
        let client_secret = client_data["secret"].as_str().unwrap();
        
        eprintln!("OAuth client created: {}", client_id);
        
        // Set up OAuth2 client using oauth2 crate
        // OAuth endpoints are under /api/v3 as well
        let oauth_client = BasicClient::new(
            ClientId::new(client_id.to_string())
        )
        .set_client_secret(ClientSecret::new(client_secret.to_string()))
        .set_auth_uri(AuthUrl::new(format!("{}/oauth2/authorize", api_base_url))?)
        .set_token_uri(TokenUrl::new(format!("{}/oauth2/token", api_base_url))?);
        
        // Generate authorization URL
        let (auth_url, csrf_state) = oauth_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("read".to_string()))
            .add_scope(Scope::new("write".to_string()))
            .url();
        
        eprintln!("Auth URL: {}", auth_url);
        
        // Programmatically authorize by GETting the auth page
        let auth_res = client
            .get(auth_url.as_str())
            .send()
            .await?;
        
        eprintln!("Authorization response status: {}", auth_res.status());
        
        // Extract authorization code based on response
        let code = if auth_res.status().is_redirection() {
            // 302 redirect
            let location = auth_res.headers().get("location")
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| anyhow::anyhow!("No location header in redirect"))?;
            
            eprintln!("Redirect location: {}", location);
            
            // Check if this is the consent page or callback
            if location.contains("/consent") {
                // Redirected to consent page - use /oauth2/authorize/decide API
                eprintln!("Redirected to consent page, submitting approval via /oauth2/authorize/decide...");
                
                // POST to /oauth2/authorize/decide endpoint (under /api/v3)
                let decide_url = format!("{}/oauth2/authorize/decide", api_base_url);
                eprintln!("Decide URL: {}", decide_url);
                
                // Submit approval with submit=approve
                let approve_res = client
                    .post(&decide_url)
                    .form(&[("submit", "approve")])
                    .send()
                    .await?;
                
                eprintln!("Approval response status: {}", approve_res.status());
                
                if !approve_res.status().is_redirection() {
                    let error_body = approve_res.text().await?;
                    eprintln!("Approval error: {}", error_body);
                    anyhow::bail!("Consent approval failed: expected redirect");
                }
                
                // Extract code from redirect after approval
                let location = approve_res.headers().get("location")
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| anyhow::anyhow!("No location header after approval"))?;
                
                eprintln!("Callback location: {}", location);
                
                let callback_url = url::Url::parse(location)?;
                callback_url
                    .query_pairs()
                    .find(|(key, _)| key == "code")
                    .map(|(_, value)| value.to_string())
                    .ok_or_else(|| anyhow::anyhow!("No code in callback URL"))?
            } else {
                // Direct redirect to callback with code
                let callback_url = url::Url::parse(location)?;
                callback_url
                    .query_pairs()
                    .find(|(key, _)| key == "code")
                    .map(|(_, value)| value.to_string())
                    .ok_or_else(|| anyhow::anyhow!("No code in callback URL"))?
            }
        } else if auth_res.status() == 200 {
            // 200 OK - consent page returned, need to submit approval
            eprintln!("Got consent page, submitting approval...");
            
            let approve_res = client
                .post(auth_url.as_str())
                .form(&[
                    ("decision", "approve"),
                    ("state", csrf_state.secret()),
                ])
                .send()
                .await?;
            
            eprintln!("Approval response status: {}", approve_res.status());
            
            let location = approve_res.headers().get("location")
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| anyhow::anyhow!("No location header after approval"))?;
            
            let callback_url = url::Url::parse(location)?;
            callback_url
                .query_pairs()
                .find(|(key, _)| key == "code")
                .map(|(_, value)| value.to_string())
                .ok_or_else(|| anyhow::anyhow!("No code in callback URL"))?
        } else {
            anyhow::bail!("Unexpected authorization response: {}", auth_res.status());
        };
        
        eprintln!("Got authorization code");
        
        // Exchange code for token
        let http_client = reqwest::Client::new();
        let token_result = oauth_client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(&http_client)
            .await?;
        
        let access_token = token_result.access_token().secret().to_string();
        eprintln!("Got access token!");
        
        Ok(access_token)
    }
    
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
    
    pub fn admin_token(&self) -> &str {
        self.admin_token.as_deref().unwrap_or("")
    }
}

// DockerCompose automatically cleaned up by Ryuk when dropped
