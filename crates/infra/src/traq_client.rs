use anyhow::Result;
use domain::{model::Message, traq_client::TraqClient};
use time::{OffsetDateTime, macros};
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
        let req_time_format =
            macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");
        let search_result = message_api::search_messages(
            &config,
            None,
            Some(since.format(&req_time_format).unwrap()),
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
