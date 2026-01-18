use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur in repository operations
#[derive(Error, Debug, PartialEq)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    Database(String),

    #[error("serialization error: {0}")]
    Serialization(String),
}

/// Errors that can occur when communicating with traQ
#[derive(Error, Debug, PartialEq)]
pub enum TraqClientError {
    #[error("HTTP request failed: {0}")]
    HttpRequest(String),

    #[error("failed to parse response: {0}")]
    ResponseParse(String),

    #[error("traQ API error: {0}")]
    ApiError(String),
}

/// Domain-level errors for service operations
#[derive(Error, Debug, PartialEq)]
pub enum DomainError {
    #[error("no valid token found to fetch user from traQ")]
    NoTokenForUserFetch,

    #[error("no valid token found to fetch user icon from traQ")]
    NoTokenForUserIcon,

    #[error("no valid token found to fetch stamp from traQ")]
    NoTokenForStampFetch,

    #[error("no valid token found to fetch stamp image from traQ")]
    NoTokenForStampImage,

    #[error("no valid token found to fetch stamps from traQ")]
    NoTokenForStampsList,

    #[error("no valid token found for user {0}")]
    NoTokenForUser(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),

    #[error(transparent)]
    TraqClient(#[from] TraqClientError),
}
