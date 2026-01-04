use anyhow::Result;

use crate::{model::Message, repository::Repository};

#[derive(Clone, Debug)]
pub struct TimelineService {
    repo: Repository,
}

impl TimelineService {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub async fn get_recommended_messages(&self) -> Result<Vec<Message>> {
        let messages = self.repo.message.find_recent_messages().await?;

        Ok(messages)
    }
}
