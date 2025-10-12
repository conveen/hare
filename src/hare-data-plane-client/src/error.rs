#[derive(Debug, thiserror::Error)]
pub enum DataPlaneError {
    /// A resource already exists.
    #[error("Resource already exists: {resource_id}")]
    AlreadyExists { resource_id: String },

    /// An error decoding bytes as Base64.
    #[error(transparent)]
    Base64Decode(#[from] base64::DecodeError),

    /// A database connection environment variable is not set.
    #[error(transparent)]
    DatabaseConnection(#[from] std::env::VarError),

    /// A precondition is not met.
    #[error("Failed precondition: {message}")]
    FailedPrecondition { message: String },

    /// An argument did not meet one or more requirements.
    #[error("Argument requirement not met: {message}")]
    InvalidArgument { message: String },

    /// A URL is not valid according to [`url::Url::parse`].
    #[error("Invalid URL for shortcut: {message}")]
    InvalidUrl { message: String },

    /// SQL migrations failed to execute.
    #[cfg(any(feature = "sqlite", feature = "postgres"))]
    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),

    /// A resource does not exist.
    #[error("Resource not found: {resource_id}")]
    NotFound { resource_id: String },

    /// A client exceeded its request limit.
    #[error("Request limit exceeded, try again shortly")]
    ResourceExhausted,

    /// A JSON (de)serialization error.
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// A SQLx error.
    #[cfg(any(feature = "sqlite", feature = "postgres"))]
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl From<DataPlaneError> for tonic::Status {
    fn from(err: DataPlaneError) -> Self {
        match err {
            DataPlaneError::AlreadyExists { resource_id } => Self::already_exists(resource_id),
            DataPlaneError::Base64Decode(_) => Self::invalid_argument("Invalid continuation token"),
            DataPlaneError::DatabaseConnection(_) => Self::internal("Internal error"),
            DataPlaneError::FailedPrecondition { message } => Self::failed_precondition(message),
            DataPlaneError::InvalidArgument { message } => Self::invalid_argument(message),
            DataPlaneError::InvalidUrl { message: _ } => Self::invalid_argument("Invalid URL for shortcut"),
            #[cfg(any(feature = "sqlite", feature = "postgres"))]
            DataPlaneError::Migration(_) => Self::internal("Internal error"),
            DataPlaneError::NotFound { resource_id } => Self::not_found(resource_id),
            DataPlaneError::ResourceExhausted => {
                Self::resource_exhausted(DataPlaneError::ResourceExhausted.to_string())
            },
            DataPlaneError::SerdeJson(_) => Self::internal("Internal error"),
            #[cfg(any(feature = "sqlite", feature = "postgres"))]
            DataPlaneError::Sqlx(_) => Self::internal("Internal error"),
        }
    }
}

pub type DataPlaneResult<T> = Result<T, DataPlaneError>;
