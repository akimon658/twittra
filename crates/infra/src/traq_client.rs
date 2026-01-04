use anyhow::Result;
use domain::{model::Message, traq_client::TraqClient};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use traq::apis::{configuration::Configuration, message_api};

pub struct TraqClientImpl {}

#[async_trait::async_trait]
impl TraqClient for TraqClientImpl {
    async fn fetch_messages_since(
        &self,
        token: &str,
        since: OffsetDateTime,
    ) -> Result<Vec<Message>> {
        let config = Configuration {
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
}
