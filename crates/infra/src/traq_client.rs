use anyhow::Result;
use domain::{model::Message, traq_client::TraqClient};
use time::{PrimitiveDateTime, macros};
use traq::apis::{configuration::Configuration, message_api};

pub struct TraqClientImpl {}

#[async_trait::async_trait]
impl TraqClient for TraqClientImpl {
    async fn fetch_messages_since(
        &self,
        token: &str,
        since: PrimitiveDateTime,
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
        let res_time_format = macros::format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]Z"
        );
        let messages = search_result
            .hits
            .into_iter()
            .map(|m| Message {
                id: m.id,
                user_id: m.user_id,
                channel_id: m.channel_id,
                content: m.content,
                created_at: PrimitiveDateTime::parse(&m.created_at, &res_time_format).unwrap(),
                updated_at: PrimitiveDateTime::parse(&m.updated_at, &res_time_format).unwrap(),
            })
            .collect();

        Ok(messages)
    }
}
