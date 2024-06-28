pub mod error;
pub mod model;
mod pagination;
mod utils;

#[cfg(feature = "sqlite")]
pub mod sqlite;

use hare_common_model::app::conveen::hare::common::pagination::{PaginationContinuation, PaginationRequest};

/// Response type for [`HareDataPlaneClient::list_shortcuts`].
///
/// Contains the list of shortcuts and a pagination continuation structure for requesting the next page of shortcuts.
pub struct ListShortcutsResponse {
    /// The list of shortcuts.
    pub shortcuts: model::CommittedShortcutList,
    /// The pagination continuation structure.
    pub pagination: PaginationContinuation,
}

#[tonic::async_trait]
pub trait HareDataPlaneClient {
    /// Bootstrap the database.
    ///
    /// Boostrap here means performing updates so it's ready to handle requests.
    /// This could mean performing migrations, adding secondary indexes, etc.
    async fn bootstrap(&self) -> error::DataPlaneResult<()>;

    /// Add one or more aliases to a shortcut.
    ///
    /// Aliases in the request are deduplicated but are not deduplicated with existing aliases, so this function is not idempotent.
    ///
    /// # Argument Requirements
    ///
    /// * `aliases` must not be empty (at least one alias to add).
    /// * `uid` must correspond to an existing destination.
    ///
    /// # Returns
    ///
    /// (optional) The list of aliases added to the shortcut.
    ///
    /// # Errors
    ///
    /// * [INVALID_ARGUMENT](enum@tonic::Code#variant.InvalidArgument): if one or more arguments do not meet the requirements.
    /// * [ALREADY_EXISTS](enum@tonic::Code#variant.AlreadyExists): if one or more aliases already exist.
    /// * [NOT_FOUND](enum@tonic::Code#variant.NotFound): if the shortcut does not exist.
    async fn add_aliases_for_shortcut(
        &self,
        uid: &str,
        aliases: &[&str],
    ) -> error::DataPlaneResult<Option<Vec<String>>>;

    /// Create a new shortcut.
    ///
    /// Shortcuts are deduplicated by URL, thus this function is idempotent.
    /// However, depending on the URL matching logic, it may be possible to submit
    /// different URLs that are semantically identical (e.g. by changing URL parameter order).
    ///
    /// # Argument Requirements
    ///
    /// * `url` must be an HTTP URL compliant with [RFC-1738](https://www.rfc-editor.org/rfc/rfc1738#section-3.3)
    /// (using the `http` or `https` scheme).
    /// * `aliases` must not be empty (at least one alias to add).
    ///
    /// # Returns
    ///
    /// The `uid` of the created shortcut's destination.
    ///
    /// # Errors
    ///
    /// * [INVALID_ARGUMENT](enum@tonic::Code#variant.InvalidArgument): if one or more arguments do not meet the requirements.
    /// * [ALREADY_EXISTS](enum@tonic::Code#variant.AlreadyExists): if one or more aliases already exist for other shortcut(s).
    async fn create_shortcut(
        &self,
        url: &str,
        is_fallback: bool,
        is_default_fallback: bool,
        description: &str,
        aliases: &[&str],
    ) -> error::DataPlaneResult<String>;

    /// Delete one or more aliases for a shortcut.
    ///
    /// If one or more aliases does not exist for a shortcut the function will exit successfully (no error raised).
    ///
    /// # Preconditions
    ///
    /// * Shortcuts must always have at least one alias.
    ///
    /// # Argument Requirements
    ///
    /// * `uid` must correspond to an existing destination.
    ///
    /// # Returns
    ///
    /// (optional) The list of aliases deleted from the shortcut.
    ///
    /// # Errors
    ///
    /// * [INVALID_ARGUMENT](enum@tonic::Code#variant.InvalidArgument): if one or more arguments do not meet the requirements.
    /// * [FAILED_PRECONDITION](enum@tonic::Code#variant.FailedPrecondition): if the shortcut only has one existing alias.
    /// * [NOT_FOUND](enum@tonic::Code#variant.NotFound): if the shortcut does not exist.
    async fn delete_aliases_for_shortcut(
        &self,
        uid: &str,
        aliases: &[&str],
    ) -> error::DataPlaneResult<Option<Vec<String>>>;

    /// Delete a shortcut.
    ///
    /// # Argument Requirements:
    ///
    /// * `uid` must correspond to an existing destination.
    ///
    /// # Errors
    ///
    /// * [INVALID_ARGUMENT](enum@tonic::Code#variant.InvalidArgument): if one or more arguments do not meet the requirements.
    /// * [NOT_FOUND](enum@tonic::Code#variant.NotFound): if the shortcut does not exist.
    async fn delete_shortcut(&self, uid: &str) -> error::DataPlaneResult<()>;

    /// Get the default fallback shortcut.
    ///
    /// # Returns
    ///
    /// The corresponding shortcut.
    ///
    /// # Errors
    ///
    /// * [NOT_FOUND](enum@tonic::Code#variant.NotFound): if the shortcut does not exist.
    async fn get_default_fallback_shortcut(&self) -> error::DataPlaneResult<model::CommittedShortcut>;

    /// Get shortcut by UID.
    ///
    /// # Argument Requirements
    ///
    /// * `uid` must correspond to an existing destination.
    ///
    /// # Returns
    ///
    /// The corresponding shortcut.
    ///
    /// # Errors
    ///
    /// * [INVALID_ARGUMENT](enum@tonic::Code#variant.InvalidArgument): if one or more arguments do not meet the requirements.
    /// * [NOT_FOUND](enum@tonic::Code#variant.NotFound): if the shortcut does not exist.
    async fn get_shortcut_by_uid(&self, uid: &str) -> error::DataPlaneResult<model::CommittedShortcut>;

    /// Get shortcut by alias.
    ///
    /// # Argument Requirements
    ///
    /// * `alias` must correspond to an existing shortcut.
    ///
    /// # Returns
    ///
    /// The corresponding shortcut.
    ///
    /// # Errors
    ///
    /// * [INVALID_ARGUMENT](enum@tonic::Code#variant.InvalidArgument): if one or more arguments do not meet the requirements.
    /// * [NOT_FOUND](enum@tonic::Code#variant.NotFound): if the shortcut does not exist.
    async fn get_shortcut_by_alias(&self, alias: &str) -> error::DataPlaneResult<model::CommittedShortcut>;

    /// Get a shortcut by UID or alias, or the default fallback shortcut.
    ///
    /// Order of precedence is `default_fallback`, `uid`, then `alias`.
    ///
    /// # Argument Requirements
    ///
    /// * Exactly one of `uid`, `alias`, or the `default_fallback` flag must be provided.
    /// * `uid` must correspond to an existing destination.
    /// * `alias` must correspond to an existing shortcut.
    ///
    /// # Returns
    ///
    /// The corresponding shortcut.
    ///
    /// # Errors
    ///
    /// * [INVALID_ARGUMENT](enum@tonic::Code#variant.InvalidArgument): if one or more arguments do not meet the requirements.
    /// * [NOT_FOUND](enum@tonic::Code#variant.NotFound): if the shortcut does not exist.
    async fn get_shortcut(
        &self,
        uid: Option<&str>,
        alias: Option<&str>,
        default_fallback: bool,
    ) -> error::DataPlaneResult<model::CommittedShortcut> {
        if default_fallback {
            self.get_default_fallback_shortcut().await
        } else if let Some(uid) = uid {
            self.get_shortcut_by_uid(uid).await
        } else if let Some(alias) = alias {
            self.get_shortcut_by_alias(alias).await
        } else {
            Err(error::DataPlaneError::InvalidArgument {
                message: "Must either set default_fallback or provide one of uid or alias".to_string(),
            })
        }
    }

    /// List all shorcuts.
    ///
    /// This function is paginated.
    /// The [page_token](struct@PaginationRequest#structfield.page_token) format is opaque to the caller and set by the server. Callers must not depend on the format.
    /// The default and maximum values for [page_size](struct@PaginationRequest#structfield.page_size) are determined by the implementation.
    /// Once all shortcuts have been returned, [next_page_token](struct@PaginationContinuation#structfield.next_continuation_token) will be empty ([None](enum@std::option::Option)).
    ///
    /// # Argument Requirements
    ///
    /// * [pagination.page_token](struct@PaginationRequest#structfield.page_token) must be a valid token provided by the server.
    /// * [pagination.page_size](struct@PaginationRequest#structfield.page_size) must be <= the maximum set by the server.
    ///
    /// # Returns
    ///
    /// The (paginated) list of shortcuts and continuation token.
    ///
    /// # Errors
    ///
    /// * [INVALID_ARGUMENT](enum@tonic::Code#variant.InvalidArgument): if one or more arguments do not meet the requirements.
    async fn list_shortcuts(&self, pagination: &PaginationRequest) -> error::DataPlaneResult<ListShortcutsResponse>;

    /// Update a shortcut.
    ///
    /// [num_params](struct@model::CommittedDestination#structfield.num_params) cannot be set directly, it is generated from `url`.
    ///
    /// Argument Requirements
    ///
    /// * One of `uid` or `alias` must be supplied and correspond to an existing destination.
    ///
    /// # Errors
    ///
    /// * [INVALID_ARGUMENT](enum@tonic::Code#variant.InvalidArgument): if one or more arguments do not meet the requirements.
    /// * [NOT_FOUND](enum@tonic::Code#variant.NotFound): if the shortcut does not exist.
    async fn update_shortcut(
        &self,
        uid: Option<&str>,
        alias: Option<&str>,
        url: Option<&str>,
        is_fallback: Option<bool>,
        is_default_fallback: Option<bool>,
        description: Option<&str>,
    ) -> error::DataPlaneResult<Option<model::CommittedShortcut>>;
}
