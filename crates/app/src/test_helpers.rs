//! Shared test utilities for app crate tests

use crate::handler::AppState;
use crate::session::{AuthSession, Backend, BasicClientSet, UserSession};
use axum::http::StatusCode;
use axum_login::AuthManagerLayerBuilder;
use domain::error::RepositoryError;
use domain::service::{MockTimelineService, MockTraqService};
use domain::{
    model::User,
    repository::UserRepository,
    service::{TimelineService, TraqService},
};
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use std::sync::Arc;
use tower_sessions::{MemoryStore, SessionManagerLayer};
use uuid::Uuid;

// For auth backend, we need a minimal UserRepository mock
mockall::mock! {
    #[derive(Debug)]
    UserRepo {}

    #[async_trait::async_trait]
    impl UserRepository for UserRepo {
        async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>, RepositoryError>;
        async fn find_random_valid_token(&self) -> Result<Option<String>, RepositoryError>;
        async fn find_token_by_user_id(&self, user_id: &Uuid) -> Result<Option<String>, RepositoryError>;
        async fn save(&self, user: &User) -> Result<(), RepositoryError>;
        async fn save_token(&self, user_id: &Uuid, access_token: &str) -> Result<(), RepositoryError>;
    }
}

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

/// Builder for creating test applications that use service mocks
///
/// This builder provides a fluent API for configuring test applications with custom
/// mock services. Any service not explicitly set will use a default mock.
///
/// # Example
///
/// ```rust
/// let app = TestAppBuilder::new()
///     .with_traq_service(mock_traq_service)
///     .with_user(user)
///     .build();
/// ```
pub struct TestAppBuilder {
    traq_service: Option<Arc<dyn TraqService>>,
    timeline_service: Option<Arc<dyn TimelineService>>,
    user: Option<User>,
}

impl TestAppBuilder {
    /// Create a new builder with all services unset (will use defaults)
    pub fn new() -> Self {
        Self {
            traq_service: None,
            timeline_service: None,
            user: None,
        }
    }

    /// Set a custom TraqService (default: MockTraqService::new())
    pub fn with_traq_service<T: TraqService + 'static>(mut self, service: T) -> Self {
        self.traq_service = Some(Arc::new(service));
        self
    }

    /// Set a custom TimelineService (default: MockTimelineService::new())
    pub fn with_timeline_service<T: TimelineService + 'static>(mut self, service: T) -> Self {
        self.timeline_service = Some(Arc::new(service));
        self
    }

    /// Set the authenticated user for this test app
    pub fn with_user(mut self, user: User) -> Self {
        self.user = Some(user);
        self
    }

    /// Build the test app using production route definitions
    pub fn build(self) -> axum::Router {
        // Use provided services or create default mocks
        let traq_service = self
            .traq_service
            .unwrap_or_else(|| Arc::new(MockTraqService::new()));
        let timeline_service = self
            .timeline_service
            .unwrap_or_else(|| Arc::new(MockTimelineService::new()));

        let state = AppState::new(traq_service, timeline_service);

        // Use production route setup
        let (router, _openapi) = crate::setup_openapi_routes();

        // Create test-specific auth and session layers
        let mock_user_repo = Arc::new(MockUserRepo::new());
        let backend = Backend::new(
            create_dummy_oauth_client(),
            "http://dummy".to_string(),
            mock_user_repo,
        );
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
