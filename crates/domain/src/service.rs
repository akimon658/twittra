use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    model::{MessageListItem, User},
    repository::Repository,
    traq_client::TraqClient,
};

#[derive(Clone, Debug)]
pub struct TimelineService {
    repo: Repository,
}

impl TimelineService {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub async fn get_recommended_messages(&self) -> Result<Vec<MessageListItem>> {
        let messages = self.repo.message.find_recent_messages().await?;

        Ok(messages)
    }
}

#[derive(Clone, Debug)]
pub struct UserService {
    repo: Repository,
    traq_client: Arc<dyn TraqClient>,
}

impl UserService {
    pub fn new(repo: Repository, traq_client: Arc<dyn TraqClient>) -> Self {
        Self { repo, traq_client }
    }

    pub async fn get_user_by_id(&self, user_id: &Uuid) -> Result<User> {
        let user = match self.repo.user.find_by_id(user_id).await? {
            Some(user) => user,
            None => {
                let token = match self.repo.user.find_random_valid_token().await? {
                    Some(token) => token,
                    None => {
                        return Err(anyhow::anyhow!(
                            "no valid token found to fetch user from traq"
                        ));
                    }
                };
                let user = self.traq_client.get_user(&token, user_id).await?;

                self.repo.user.save(&user).await?;

                user
            }
        };

        Ok(user)
    }
}
