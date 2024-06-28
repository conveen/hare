#[derive(Debug, thiserror::Error)]
pub enum DataPlaneError {
    #[error("Resource already exists: {resource_id}")]
    AlreadyExists { resource_id: String },

    #[error(transparent)]
    Base64Decode(#[from] base64::DecodeError),

    #[error("Failed precondition: {message}")]
    FailedPrecondition { message: String },

    #[error("Argument requirement not met: {message}")]
    InvalidArgument { message: String },

    #[error("Invalid URL for shortcut: {message}")]
    InvalidUrl { message: String },

    #[cfg(feature = "sqlite")]
    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Resource not found: {resource_id}")]
    NotFound { resource_id: String },

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[cfg(feature = "sqlite")]
    #[error(transparent)]
    Sqlite(#[from] sqlx::Error),
}

impl From<DataPlaneError> for tonic::Status {
    fn from(err: DataPlaneError) -> Self {
        match err {
            DataPlaneError::AlreadyExists { resource_id } => Self::already_exists(resource_id),
            DataPlaneError::Base64Decode(_) => Self::invalid_argument("Invalid continuation token"),
            DataPlaneError::FailedPrecondition { message } => Self::failed_precondition(message),
            DataPlaneError::InvalidArgument { message } => Self::invalid_argument(message),
            DataPlaneError::InvalidUrl { message: _ } => Self::invalid_argument("Invalid URL for shortcut"),
            DataPlaneError::Migration(_) => Self::internal("Internal error"),
            DataPlaneError::NotFound { resource_id } => Self::not_found(resource_id),
            DataPlaneError::Serde(_) => Self::internal("Internal error"),
            DataPlaneError::Sqlite(_) => Self::internal("Internal error"),
        }
    }
}

pub type DataPlaneResult<T> = Result<T, DataPlaneError>;
