pub mod error;
pub mod model;
mod pagination;
mod relational;
mod utils;

#[cfg(feature = "postgres")]
pub mod postgres {
    pub use crate::relational::HareDataPlanePostgres;
}
#[cfg(feature = "sqlite")]
pub mod sqlite {
    pub use crate::relational::HareDataPlaneSqlite;
}

use hare_common_model::pagination::{PaginationContinuation, PaginationRequest};

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
    async fn bootstrap(&self) -> error::DataPlaneResult<()> {
        todo!();
    }

    /// Add one or more aliases to a shortcut.
    ///
    /// # Argument Requirements
    ///
    /// * `aliases` must not be empty (at least one alias to add).
    /// * `uid` must correspond to an existing destination.
    /// * Aliases must be unique.
    ///
    /// # Returns
    ///
    /// (optional) The list of aliases added to the shortcut.
    ///
    /// # Errors
    ///
    /// * [`error::DataPlaneError::InvalidArgument`]: if one or more arguments do not meet the requirements.
    /// * [`error::DataPlaneError::AlreadyExists`]: if one or more aliases already exist.
    /// * [`error::DataPlaneError::NotFound`]: if the shortcut does not exist.
    #[allow(unused_variables)]
    async fn add_aliases_for_shortcut(
        &self,
        uid: &str,
        aliases: &[&str],
    ) -> error::DataPlaneResult<Option<Vec<model::CommittedAlias>>> {
        todo!();
    }

    /// Create a new shortcut.
    ///
    /// # Argument Requirements
    ///
    /// * `url` must be an HTTP URL compliant with [RFC-1738](https://www.rfc-editor.org/rfc/rfc1738#section-3.3)
    /// (using the `http` or `https` scheme).
    /// * `aliases` must not be empty (at least one alias to add).
    /// * Aliases must be unique.
    ///
    /// # Returns
    ///
    /// The created shortcut.
    ///
    /// # Errors
    ///
    /// * [`error::DataPlaneError::InvalidArgument`]: if one or more arguments do not meet the requirements.
    /// * [`error::DataPlaneError::AlreadyExists`]: if a shortcut with the same URL Or one or more aliases already exist.
    #[allow(unused_variables)]
    async fn create_shortcut(
        &self,
        url: &str,
        is_fallback: bool,
        description: &str,
        aliases: &[&str],
    ) -> error::DataPlaneResult<model::CommittedShortcut> {
        todo!();
    }

    /// Delete one or more aliases for a shortcut.
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
    /// * [`error::DataPlaneError::InvalidArgument`]: if one or more arguments do not meet the requirements.
    /// * [`error::DataPlaneError::FailedPrecondition`]: if the shortcut only has one existing alias.
    /// * [`error::DataPlaneError::NotFound`]: if the shortcut or one of the aliases do not exist.
    #[allow(unused_variables)]
    async fn delete_aliases_for_shortcut(
        &self,
        uid: &str,
        aliases: &[&str],
    ) -> error::DataPlaneResult<Option<Vec<model::CommittedAlias>>> {
        todo!();
    }

    /// Delete a shortcut.
    ///
    /// # Argument Requirements:
    ///
    /// * `uid` must correspond to an existing destination.
    ///
    /// # Errors
    ///
    /// * [`error::DataPlaneError::InvalidArgument`]: if one or more arguments do not meet the requirements.
    /// * [`error::DataPlaneError::NotFound`]: if the shortcut does not exist.
    #[allow(unused_variables)]
    async fn delete_shortcut(&self, uid: &str) -> error::DataPlaneResult<()> {
        todo!();
    }

    /// Get the default fallback shortcut.
    ///
    /// # Returns
    ///
    /// The corresponding shortcut.
    ///
    /// # Errors
    ///
    /// * [`error::DataPlaneError::NotFound`]: if the shortcut does not exist.
    #[allow(unused_variables)]
    async fn get_default_fallback_shortcut(&self) -> error::DataPlaneResult<model::CommittedShortcut> {
        todo!();
    }

    /// Set the default fallback shortcut.
    ///
    /// Atomically sets an existing shorcut as the default fallback and unsets the existing defaul fallback.
    ///
    /// # Preconditions
    ///
    /// There must always be at most one (1) default fallback shortcut.
    ///
    /// # Errors
    ///
    /// * [`error::DataPlaneError::NotFound`]: if the shortcut does not exist.
    #[allow(unused_variables)]
    async fn set_default_fallback_shortcut(&self, uid: &str) -> error::DataPlaneResult<()> {
        todo!();
    }

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
    /// * [`error::DataPlaneError::InvalidArgument`]: if one or more arguments do not meet the requirements.
    /// * [`error::DataPlaneError::NotFound`]: if the shortcut does not exist.
    #[allow(unused_variables)]
    async fn get_shortcut_by_uid(&self, uid: &str) -> error::DataPlaneResult<model::CommittedShortcut> {
        todo!();
    }

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
    /// * [`error::DataPlaneError::InvalidArgument`]: if one or more arguments do not meet the requirements.
    /// * [`error::DataPlaneError::NotFound`]: if the shortcut does not exist.
    #[allow(unused_variables)]
    async fn get_shortcut_by_alias(&self, alias: &str) -> error::DataPlaneResult<model::CommittedShortcut> {
        todo!();
    }

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
    /// * [`error::DataPlaneError::InvalidArgument`]: if one or more arguments do not meet the requirements.
    /// * [`error::DataPlaneError::NotFound`]: if the shortcut does not exist.
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
    /// The [`PaginationRequest#structfield.continuation_token`] format is opaque to the caller and set by the server. Callers must not depend on the format.
    /// The default and maximum values for [`PaginationRequest#structfield.page_size`] are determined by the implementation.
    /// Once all shortcuts have been returned, [`PaginationContinuation#structfield.next_continuation_token`] will be empty ([None](enum@std::option::Option)).
    ///
    /// # Argument Requirements
    ///
    /// * [`PaginationRequest#structfield.continuation_token`] must be a valid token provided by the server.
    /// * [`PaginationRequest#structfield.page_size`] must be <= the maximum set by the server.
    ///
    /// # Returns
    ///
    /// The list of shortcuts and continuation token.
    ///
    /// # Errors
    ///
    /// * [`error::DataPlaneError::InvalidArgument`]: if one or more arguments do not meet the requirements.
    #[allow(unused_variables)]
    async fn list_shortcuts(&self, pagination: &PaginationRequest) -> error::DataPlaneResult<ListShortcutsResponse> {
        todo!();
    }

    /// Update a shortcut.
    ///
    /// Argument Requirements
    ///
    /// * `uid` must correspond to an existing destination.
    ///
    /// # Errors
    ///
    /// * [`error::DataPlaneError::InvalidArgument`]: if one or more arguments do not meet the requirements.
    /// * [`error::DataPlaneError::NotFound`]: if the shortcut does not exist.
    #[allow(unused_variables)]
    async fn update_shortcut(
        &self,
        uid: &str,
        url: Option<&str>,
        is_fallback: Option<bool>,
        description: Option<&str>,
    ) -> error::DataPlaneResult<model::CommittedShortcut> {
        todo!();
    }
}
