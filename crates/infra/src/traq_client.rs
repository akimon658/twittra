use anyhow::Result;
use domain::{
    model::{Message, User},
    traq_client::TraqClient,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use traq::apis::{configuration::Configuration, message_api, user_api};
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
    ) -> Result<Vec<Message>> {
        let config = Configuration {
            base_path: self.base_url.clone(),
            oauth_access_token: Some(token.to_string()),
            ..Default::default()
        };
        let search_result = message_api::search_messages(
            &config,
            None,
            Some(since.format(&Rfc3339)?),
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
        .await?;
        let messages = search_result
            .hits
            .into_iter()
            .map(|msg| msg.try_into())
            .collect::<Result<Vec<Message>, _>>()?;

        Ok(messages)
    }

    async fn get_user(&self, token: &str, user_id: &Uuid) -> Result<User> {
        let config = Configuration {
            base_path: self.base_url.clone(),
            oauth_access_token: Some(token.to_string()),
            ..Default::default()
        };
        let traq_user = user_api::get_user(&config, &user_id.to_string()).await?;
        let user = traq_user.into();

        Ok(user)
    }

    async fn get_user_icon(&self, token: &str, user_id: &Uuid) -> Result<(Vec<u8>, String)> {
        let config = Configuration {
            base_path: self.base_url.clone(),
            oauth_access_token: Some(token.to_string()),
            ..Default::default()
        };
        let response = user_api::get_user_icon(&config, &user_id.to_string()).await?;
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        let bytes = response.bytes().await?.to_vec();
        Ok((bytes, content_type))
    }
}
