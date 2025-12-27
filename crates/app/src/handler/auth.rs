use std::{
    fmt::{self, Debug, Formatter},
    result,
    sync::Arc,
};

use axum::{
    Router,
    extract::Query,
    response::{IntoResponse, Redirect},
    routing,
};
use axum_login::{AuthUser, AuthnBackend};
use domain::{model::User, repository::UserRepository};
use http::StatusCode;
use oauth2::{
    AsyncHttpClient, AuthorizationCode, CsrfToken, EndpointNotSet, EndpointSet, TokenResponse,
    basic::{BasicClient, BasicRequestTokenError},
    url::Url,
};
use reqwest::Client;
use traq::apis::{
    self,
    configuration::Configuration,
    me_api::{self, GetMeError},
};
use uuid::Uuid;

use crate::handler::AppState;

#[derive(Clone)]
pub struct UserSession {
    pub id: Uuid,
}

impl Debug for UserSession {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserSession")
            .field("id", &self.id)
            .field("access_token", &"****")
            .finish()
    }
}

impl AuthUser for UserSession {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        &[]
    }
}

type BasicClientSet =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

#[derive(Clone)]
pub struct Backend {
    http_client: Client,
    oauth_client: BasicClientSet,
    user_repository: Arc<dyn UserRepository>,
}

impl Backend {
    pub fn new(oauth_client: BasicClientSet, user_repository: Arc<dyn UserRepository>) -> Self {
        Self {
            http_client: Client::new(),
            oauth_client,
            user_repository,
        }
    }

    fn authorize_url(&self) -> (Url, CsrfToken) {
        self.oauth_client.authorize_url(CsrfToken::new_random).url()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error(transparent)]
    Oauth2(BasicRequestTokenError<<reqwest::Client as AsyncHttpClient<'static>>::Error>),
    #[error(transparent)]
    UserRepository(anyhow::Error),
    #[error(transparent)]
    Traq(apis::Error<GetMeError>),
}

impl AuthnBackend for Backend {
    type User = UserSession;
    type Credentials = String;
    type Error = BackendError;

    async fn authenticate(
        &self,
        code: Self::Credentials,
    ) -> result::Result<Option<Self::User>, Self::Error> {
        let token_res = self
            .oauth_client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(&self.http_client)
            .await
            .map_err(Self::Error::Oauth2)?;
        let config = Configuration {
            oauth_access_token: Some(token_res.access_token().secret().to_string()),
            ..Default::default()
        };
        let traq_user = me_api::get_me(&config).await.map_err(Self::Error::Traq)?;
        let user = User {
            id: traq_user.id,
            handle: traq_user.name,
        };

        self.user_repository
            .save_user(&user)
            .await
            .map_err(Self::Error::UserRepository)?;
        self.user_repository
            .save_token(&traq_user.id, token_res.access_token().secret())
            .await
            .map_err(Self::Error::UserRepository)?;

        Ok(Some(UserSession { id: traq_user.id }))
    }

    async fn get_user(
        &self,
        user_id: &axum_login::UserId<Self>,
    ) -> result::Result<Option<Self::User>, Self::Error> {
        Ok(Some(UserSession { id: *user_id }))
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;

const CSRF_STATE_KEY: &str = "oauth.csrf_state";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/login", routing::get(login))
        .route("/auth/callback", routing::get(oauth_callback))
}

pub async fn login(auth_session: AuthSession) -> impl IntoResponse {
    let (authorize_url, csrf_state) = auth_session.backend.authorize_url();

    auth_session
        .session
        .insert(CSRF_STATE_KEY, csrf_state.secret())
        .await
        .expect("Failed to store CSRF state in session");

    Redirect::to(authorize_url.as_str())
}

#[derive(serde::Deserialize)]
pub struct AuthorizeQuery {
    code: String,
    state: String,
}

pub async fn oauth_callback(
    mut auth_session: AuthSession,
    Query(AuthorizeQuery {
        code,
        state: new_state,
    }): Query<AuthorizeQuery>,
) -> impl IntoResponse {
    let Ok(Some(old_state)) = auth_session.session.get::<String>(CSRF_STATE_KEY).await else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    if old_state != new_state {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let user = match auth_session.authenticate(code).await {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(e) => {
            eprintln!("Authentication error: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if let Err(_) = auth_session.login(&user).await {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    Redirect::to("/").into_response()
}
