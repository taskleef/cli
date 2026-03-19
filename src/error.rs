use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaskleefError {
    #[error("API key not configured. Set TASKLEEF_API_KEY or use --auth-file")]
    MissingApiKey,

    #[error("Auth file not found: {0}")]
    AuthFileNotFound(String),

    #[error("TASKLEEF_API_KEY not set in auth file")]
    ApiKeyNotInAuthFile,

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error: {0}")]
    Api(String),

    #[error("No {entity} found matching '{query}'")]
    NotFound { entity: String, query: String },

    #[error("No accessible boards found")]
    NoBoards,

    #[error("{0}")]
    Usage(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Prompt error: {0}")]
    Dialoguer(#[from] dialoguer::Error),
}

pub type Result<T> = std::result::Result<T, TaskleefError>;
