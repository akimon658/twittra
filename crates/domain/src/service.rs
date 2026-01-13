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

    pub async fn get_user_icon(&self, user_id: &Uuid) -> Result<(Vec<u8>, String)> {
        let token = match self.repo.user.find_random_valid_token().await? {
            Some(token) => token,
            None => {
                return Err(anyhow::anyhow!(
                    "no valid token found to fetch user icon from traQ"
                ));
            }
        };
        let icon = self.traq_client.get_user_icon(&token, user_id).await?;

        Ok(icon)
    }

    pub async fn get_stamp_image(&self, stamp_id: &Uuid) -> Result<(Vec<u8>, String)> {
        let token = match self.repo.user.find_random_valid_token().await? {
            Some(token) => token,
            None => {
                return Err(anyhow::anyhow!(
                    "no valid token found to fetch stamp image from traQ"
                ));
            }
        };
        let image = self.traq_client.get_stamp_image(&token, stamp_id).await?;

        Ok(image)
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

    pub async fn get_stamps(&self) -> Result<Vec<Stamp>> {
        let token = match self.repo.user.find_random_valid_token().await? {
            Some(token) => token,
            None => {
                return Err(anyhow::anyhow!(
                    "no valid token found to fetch stamps from traQ"
                ));
            }
        };
        let stamps = self.traq_client.get_stamps(&token).await?;

        // Cache all stamps
        for stamp in &stamps {
            self.repo.stamp.save(stamp).await?;
        }

        Ok(stamps)
    }

    pub async fn add_message_stamp(
        &self,
        user_id: &Uuid,
        message_id: &Uuid,
        stamp_id: &Uuid,
        count: i32,
    ) -> Result<()> {
        let token = match self.repo.user.find_token_by_user_id(user_id).await? {
            Some(token) => token,
            None => {
                return Err(anyhow::anyhow!(
                    "no valid token found for user {}",
                    user_id
                ));
            }
        };

        // 1. Add stamp to traQ
        self.traq_client
            .add_message_stamp(&token, message_id, stamp_id, count)
            .await?;

        // 2. Fetch updated message from traQ (to get latest reactions)
        let message = self.traq_client.get_message(&token, message_id).await?;

        // 3. Update local DB
        self.repo.message.save_batch(&[message]).await?;

        Ok(())
    }

    pub async fn remove_message_stamp(
        &self,
        user_id: &Uuid,
        message_id: &Uuid,
        stamp_id: &Uuid,
    ) -> Result<()> {
        let token = match self.repo.user.find_token_by_user_id(user_id).await? {
            Some(token) => token,
            None => {
                return Err(anyhow::anyhow!(
                    "no valid token found for user {}",
                    user_id
                ));
            }
        };

        // 1. Remove stamp from traQ
        self.traq_client
            .remove_message_stamp(&token, message_id, stamp_id)
            .await?;

        // 2. Optimistically update local DB by directly removing the reaction
        //    This avoids race conditions with traQ's eventual consistency
        self.repo
            .message
            .remove_reaction(message_id, stamp_id, user_id)
            .await?;

        Ok(())
    }
}
