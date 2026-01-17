//! Shared test utilities for app crate tests

use crate::session::{AuthSession, Backend, BasicClientSet, UserSession};
use axum::http::StatusCode;
use domain::{model::User, repository::UserRepository};
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use std::sync::Arc;

/// Creates a dummy OAuth client for testing purposes
pub fn create_dummy_oauth_client() -> BasicClientSet {
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
pub fn create_test_backend<R>(user_repo: Arc<R>) -> Backend
where
    R: UserRepository + Send + Sync + 'static,
{
    Backend::new(
        create_dummy_oauth_client(),
        "http://dummy".to_string(),
        user_repo,
    )
}

/// Creates a test login handler that can be used in test routers
pub fn create_test_login_handler(
    user: Option<User>,
) -> impl Fn(AuthSession) -> std::pin::Pin<Box<dyn std::future::Future<Output = StatusCode> + Send>>
+ Clone
+ Send
+ 'static {
    move |mut auth: AuthSession| {
        let user = user.clone();
        Box::pin(async move {
            if let Some(user_session) = user.map(|u| UserSession { id: u.id }) {
                auth.login(&user_session).await.unwrap();
                StatusCode::OK
            } else {
                StatusCode::UNAUTHORIZED
            }
        })
    }
}
