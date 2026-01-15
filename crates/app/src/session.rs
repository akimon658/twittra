use std::{
    fmt::{self, Debug, Formatter},
    result,
    sync::Arc,
};

use axum_login::{AuthUser, AuthnBackend};
use domain::repository::UserRepository;
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

pub type BasicClientSet =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

#[derive(Clone, Debug)]
pub struct Backend {
    http_client: Client,
    oauth_client: BasicClientSet,
    traq_base_url: String,
    user_repository: Arc<dyn UserRepository>,
}

impl Backend {
    pub fn new(
        oauth_client: BasicClientSet,
        traq_base_url: String,
        user_repository: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            http_client: Client::new(),
            oauth_client,
            traq_base_url,
            user_repository,
        }
    }

    pub fn authorize_url(&self) -> (Url, CsrfToken) {
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
            base_path: self.traq_base_url.clone(),
            oauth_access_token: Some(token_res.access_token().secret().to_string()),
            ..Default::default()
        };
        let user = me_api::get_me(&config)
            .await
            .map_err(Self::Error::Traq)?
            .into();

        self.user_repository
            .save(&user)
            .await
            .map_err(Self::Error::UserRepository)?;
        self.user_repository
            .save_token(&user.id, token_res.access_token().secret())
            .await
            .map_err(Self::Error::UserRepository)?;

        Ok(Some(UserSession { id: user.id }))
    }

    async fn get_user(
        &self,
        user_id: &axum_login::UserId<Self>,
    ) -> result::Result<Option<Self::User>, Self::Error> {
        Ok(Some(UserSession { id: *user_id }))
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;
