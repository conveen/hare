#[derive(Debug, thiserror::Error)]
pub enum DataPlaneError {
    /// A resource already exists.
    #[error("Resource already exists: {resource_id}")]
    AlreadyExists { resource_id: String },

    /// An error decoding bytes as Base64.
    #[error(transparent)]
    Base64Decode(#[from] base64::DecodeError),

    /// A DynamoDB client error.
    #[cfg(feature = "dynamodb")]
    #[error(transparent)]
    DynamoDb(#[from] aws_sdk_dynamodb::Error),

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
    #[cfg(feature = "sqlite")]
    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),

    /// A resource does not exist.
    #[error("Resource not found: {resource_id}")]
    NotFound { resource_id: String },

    /// A client exceeded its request limit.
    #[error("Request limit exceeded, try again shortly")]
    ResourceExhausted,

    /// A DynamoDB request type (de)serialization error.
    #[cfg(feature = "dynamodb")]
    #[error(transparent)]
    SerdeDynamo(#[from] serde_dynamo::Error),

    /// A JSON (de)serialization error.
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// A SQLite query error.
    #[cfg(feature = "sqlite")]
    #[error(transparent)]
    Sqlite(#[from] sqlx::Error),
}

impl From<DataPlaneError> for tonic::Status {
    fn from(err: DataPlaneError) -> Self {
        match err {
            DataPlaneError::AlreadyExists { resource_id } => Self::already_exists(resource_id),
            DataPlaneError::Base64Decode(_) => Self::invalid_argument("Invalid continuation token"),
            #[cfg(feature = "dynamodb")]
            DataPlaneError::DynamoDb(_) => Self::internal("Internal error"),
            DataPlaneError::FailedPrecondition { message } => Self::failed_precondition(message),
            DataPlaneError::InvalidArgument { message } => Self::invalid_argument(message),
            DataPlaneError::InvalidUrl { message: _ } => Self::invalid_argument("Invalid URL for shortcut"),
            #[cfg(feature = "sqlite")]
            DataPlaneError::Migration(_) => Self::internal("Internal error"),
            DataPlaneError::NotFound { resource_id } => Self::not_found(resource_id),
            DataPlaneError::ResourceExhausted => {
                Self::resource_exhausted(DataPlaneError::ResourceExhausted.to_string())
            },
            #[cfg(feature = "dynamodb")]
            DataPlaneError::SerdeDynamo(_) => Self::internal("Internal error"),
            DataPlaneError::SerdeJson(_) => Self::internal("Internal error"),
            #[cfg(feature = "sqlite")]
            DataPlaneError::Sqlite(_) => Self::internal("Internal error"),
        }
    }
}

pub type DataPlaneResult<T> = Result<T, DataPlaneError>;
