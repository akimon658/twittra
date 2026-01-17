//! Shared test utilities for app crate tests

use crate::session::{AuthSession, Backend, BasicClientSet, UserSession};
use axum::http::StatusCode;
use domain::{model::User, repository::UserRepository};
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use std::sync::Arc;

/// Creates a dummy OAuth client for testing purposes
fn create_dummy_oauth_client() -> BasicClientSet {
    let client_id = ClientId::new("dummy_id".to_string());
    let client_secret = ClientSecret::new("dummy_secret".to_string());
    let auth_url = AuthUrl::new("http://dummy".to_string()).unwrap();
    let token_url = TokenUrl::new("http://dummy".to_string()).unwrap();
    let redirect_url = RedirectUrl::new("http://dummy/callback".to_string()).unwrap();

    oauth2::basic::BasicClient::new(client_id)
        .set_client_secret(client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(redirect_url)
}

/// Creates a test Backend with dummy OAuth configuration
fn create_test_backend(user_repo: Arc<dyn UserRepository>) -> Backend {
    Backend::new(
        create_dummy_oauth_client(),
        "http://dummy".to_string(),
        user_repo,
    )
}

/// Builder for creating test applications that reuse production route definitions
///
/// This builder provides a fluent API for configuring test applications with custom
/// mock repositories. Any repository not explicitly set will use a default mock.
///
/// # Example
///
/// ```rust
/// let app = TestAppBuilder::new()
///     .with_stamp_repo(mock_stamp_repo)
///     .with_user(user)
///     .build();
/// ```
pub struct TestAppBuilder {
    message_repo: Option<Arc<dyn domain::repository::MessageRepository>>,
    stamp_repo: Option<Arc<dyn domain::repository::StampRepository>>,
    user_repo: Option<Arc<dyn domain::repository::UserRepository>>,
    traq_client: Option<Arc<dyn domain::traq_client::TraqClient>>,
    user: Option<User>,
}

impl TestAppBuilder {
    /// Create a new builder with all repositories unset (will use defaults)
    pub fn new() -> Self {
        Self {
            message_repo: None,
            stamp_repo: None,
            user_repo: None,
            traq_client: None,
            user: None,
        }
    }

    /// Set a custom message repository (default: MockMessageRepository::new())
    pub fn with_message_repo<T: domain::repository::MessageRepository + 'static>(
        mut self,
        repo: T,
    ) -> Self {
        self.message_repo = Some(Arc::new(repo));
        self
    }

    /// Set a custom stamp repository (default: MockStampRepository::new())
    pub fn with_stamp_repo<T: domain::repository::StampRepository + 'static>(
        mut self,
        repo: T,
    ) -> Self {
        self.stamp_repo = Some(Arc::new(repo));
        self
    }

    /// Set a custom user repository (default: MockUserRepository::new())
    pub fn with_user_repo<T: domain::repository::UserRepository + 'static>(
        mut self,
        repo: T,
    ) -> Self {
        self.user_repo = Some(Arc::new(repo));
        self
    }

    /// Set a custom TraqClient (default: MockTraqClient::new())
    pub fn with_traq_client<T: domain::traq_client::TraqClient + 'static>(
        mut self,
        client: T,
    ) -> Self {
        self.traq_client = Some(Arc::new(client));
        self
    }

    /// Set the authenticated user for this test app
    pub fn with_user(mut self, user: User) -> Self {
        self.user = Some(user);
        self
    }

    /// Build the test app using production route definitions
    pub fn build(self) -> axum::Router {
        use crate::handler::AppState;
        use crate::mocks::{
            MockMessageRepository, MockStampRepository, MockTraqClient, MockUserRepository,
        };
        use axum_login::AuthManagerLayerBuilder;
        use domain::repository::Repository;
        use tower_sessions::{MemoryStore, SessionManagerLayer};

        // Use provided repositories or create default mocks
        let message_repo = self
            .message_repo
            .unwrap_or_else(|| Arc::new(MockMessageRepository::new()));
        let stamp_repo = self
            .stamp_repo
            .unwrap_or_else(|| Arc::new(MockStampRepository::new()));
        let user_repo = self
            .user_repo
            .unwrap_or_else(|| Arc::new(MockUserRepository::new()));
        let traq_client = self
            .traq_client
            .unwrap_or_else(|| Arc::new(MockTraqClient::new()));

        let repo = Repository {
            message: message_repo,
            stamp: stamp_repo,
            user: user_repo.clone(),
        };

        let state = AppState::new(repo, traq_client);

        // Use production route setup
        let (router, _openapi) =
            crate::setup_openapi_routes().expect("Failed to setup OpenAPI routes");

        // Create test-specific auth and session layers
        let backend = create_test_backend(user_repo);
        let session_layer = SessionManagerLayer::new(MemoryStore::default());
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

        let user = self.user;

        // Nest routes under /api/v1, add test login endpoint, then apply auth layer to everything
        axum::Router::new()
            .nest("/api/v1", router)
            .route(
                "/login",
                axum::routing::post(|mut auth: AuthSession| async move {
                    if let Some(user_session) = user.map(|u| UserSession { id: u.id }) {
                        auth.login(&user_session).await.unwrap();
                        StatusCode::OK
                    } else {
                        StatusCode::UNAUTHORIZED
                    }
                }),
            )
            .layer(auth_layer)
            .with_state(state)
    }
}
