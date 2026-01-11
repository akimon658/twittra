use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    model::{MessageListItem, Stamp, User},
    repository::Repository,
    traq_client::TraqClient,
};

/// Service for timeline-related operations.
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

/// Handles general data fetching from traQ.
/// It utilizes the repository as a cache and fetches data from traQ only when necessary.
/// Twittra's unique features such as recommendations are not handled here.
#[derive(Clone, Debug)]
pub struct TraqService {
    repo: Repository,
    traq_client: Arc<dyn TraqClient>,
}

impl TraqService {
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
                            "no valid token found to fetch user from traQ"
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

    pub async fn get_stamp_by_id(&self, stamp_id: &Uuid) -> Result<Stamp> {
        let stamp = match self.repo.stamp.find_by_id(stamp_id).await? {
            Some(stamp) => stamp,
            None => {
                let token = match self.repo.user.find_random_valid_token().await? {
                    Some(token) => token,
                    None => {
                        return Err(anyhow::anyhow!(
                            "no valid token found to fetch stamp from traQ"
                        ));
                    }
                };
                let stamp = self.traq_client.get_stamp(&token, stamp_id).await?;

                self.repo.stamp.save(&stamp).await?;

                stamp
            }
        };

        Ok(stamp)
    }
}
