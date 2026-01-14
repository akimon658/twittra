use anyhow::Result;
use domain::{
    model::{Message, User},
    traq_client::TraqClient,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
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

    async fn get_stamp(&self, token: &str, stamp_id: &Uuid) -> Result<domain::model::Stamp> {
        let config = Configuration {
            base_path: self.base_url.clone(),
            oauth_access_token: Some(token.to_string()),
            ..Default::default()
        };
        let traq_stamp = stamp_api::get_stamp(&config, &stamp_id.to_string()).await?;
        let stamp = traq_stamp.into();

        Ok(stamp)
    }

    async fn get_stamps(&self, token: &str) -> Result<Vec<domain::model::Stamp>> {
        let config = Configuration {
            base_path: self.base_url.clone(),
            oauth_access_token: Some(token.to_string()),
            ..Default::default()
        };
        let traq_stamps = stamp_api::get_stamps(&config, None, None).await?;
        let stamps = traq_stamps.into_iter().map(|s| s.into()).collect();

        Ok(stamps)
    }

    async fn get_stamp_image(&self, token: &str, stamp_id: &Uuid) -> Result<(Vec<u8>, String)> {
        let config = Configuration {
            base_path: self.base_url.clone(),
            oauth_access_token: Some(token.to_string()),
            ..Default::default()
        };
        let response = stamp_api::get_stamp_image(&config, &stamp_id.to_string()).await?;
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        let bytes = response.bytes().await?.to_vec();
        Ok((bytes, content_type))
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

    async fn add_message_stamp(
        &self,
        token: &str,
        message_id: &Uuid,
        stamp_id: &Uuid,
        count: i32,
    ) -> Result<()> {
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
        .await?;

        Ok(())
    }

    async fn remove_message_stamp(
        &self,
        token: &str,
        message_id: &Uuid,
        stamp_id: &Uuid,
    ) -> Result<()> {
        let config = Configuration {
            base_path: self.base_url.clone(),
            oauth_access_token: Some(token.to_string()),
            ..Default::default()
        };
        message_api::remove_message_stamp(&config, &message_id.to_string(), &stamp_id.to_string())
            .await?;

        Ok(())
    }

    async fn get_message(&self, token: &str, message_id: &Uuid) -> Result<Message> {
        let config = Configuration {
            base_path: self.base_url.clone(),
            oauth_access_token: Some(token.to_string()),
            ..Default::default()
        };
        let message = message_api::get_message(&config, &message_id.to_string()).await?;
        let message = message.try_into()?;

        Ok(message)
    }
}
