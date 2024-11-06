use crate::{
    error::{self},
    model::{CommittedAlias, CommittedDestination, CommittedShortcut, CommittedShortcutList},
    pagination::ContinuationToken,
    utils, HareDataPlaneClient, ListShortcutsResponse,
};
use hare_common_model::pagination::{PaginationContinuation, PaginationRequest};
use sqlx::{error::DatabaseError, Row};

/// SQLite data plane client.
pub struct HareDataPlaneSqlite {
    codec: base64::engine::GeneralPurpose,
    connection: sqlx::Pool<sqlx::Sqlite>,
}

impl std::fmt::Debug for HareDataPlaneSqlite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HareDataPlaneSqlite").finish()
    }
}

impl HareDataPlaneSqlite {
    pub fn from_connection(connection: sqlx::Pool<sqlx::Sqlite>) -> Self {
        let codec = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        HareDataPlaneSqlite { codec, connection }
    }

    pub async fn try_from_url<U: AsRef<str>>(database_url: U) -> error::DataPlaneResult<Self> {
        let connection = sqlx::Pool::connect(database_url.as_ref())
            .await
            .map_err(error::DataPlaneError::from)
            .inspect_err(|err| {
                tracing::error!(
                    %err,
                    database_url = database_url.as_ref(),
                    "Failed to connect to database",
                );
            })?;
        Ok(Self::from_connection(connection))
    }

    /// Add one alias to a shortcut.
    async fn add_alias(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        uid: &str,
        alias: &str,
    ) -> error::DataPlaneResult<sqlx::sqlite::SqliteQueryResult> {
        sqlx::query("INSERT INTO alias (destination_uid, name) VALUES (?, ?)")
            .bind(uid)
            .bind(alias)
            .execute(&mut **tx)
            .await
            .map_err(|err| match err {
                sqlx::Error::Database(ref db_err) => {
                    if db_err.downcast_ref::<sqlx::sqlite::SqliteError>().is_foreign_key_violation() {
                        error::DataPlaneError::NotFound { resource_id: uid.to_string() }
                    } else if db_err.downcast_ref::<sqlx::sqlite::SqliteError>().is_unique_violation() {
                        error::DataPlaneError::AlreadyExists { resource_id: alias.to_string() }
                    } else {
                        let dp_err = error::DataPlaneError::from(err);
                        tracing::error!(
                            err = %dp_err,
                            destination_uid = uid,
                            "Unexpected database error when adding alias for shortcut",
                        );
                        dp_err
                    }
                },
                _ => {
                    let dp_err = error::DataPlaneError::from(err);
                    tracing::error!(
                        err = %dp_err,
                        destination_uid = uid,
                        "Unexpected error when adding alias for shortcut",
                    );
                    dp_err
                },
            })
    }

    /// Delete one alias.
    async fn delete_alias(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        uid: &str,
        alias: &str,
    ) -> error::DataPlaneResult<bool> {
        Ok(sqlx::query("DELETE FROM alias WHERE destination_uid = ? AND name = ?")
            .bind(uid)
            .bind(alias)
            .execute(&mut **tx)
            .await
            .map_err(error::DataPlaneError::from)
            .inspect_err(|err| {
                tracing::error!(
                    %err,
                    destination_uid = uid,
                    alias,
                    "Failed to delete alias for destination",
                );
            })?
            .rows_affected()
            == 1)
    }

    /// Get aliases for a destination.
    async fn get_aliases(&self, uid: &str) -> error::DataPlaneResult<Vec<CommittedAlias>> {
        sqlx::query_as!(
            CommittedAlias,
            "SELECT destination_uid, NULL AS \"uid?: String\", name FROM alias WHERE destination_uid = ?",
            uid,
        )
        .fetch_all(&self.connection)
        .await
        .map_err(error::DataPlaneError::from)
        .inspect_err(|err| {
            tracing::error!(
                %err,
                destination_uid = uid,
                "Failed to get aliases for destination",
            );
        })
    }

    /// Get destination by UID or the default fallback.
    ///
    /// If no destination UID is provided the default fallback is returned, if defined.
    async fn get_destination(&self, maybe_uid: Option<&str>) -> error::DataPlaneResult<CommittedDestination> {
        if let Some(uid) = maybe_uid {
            sqlx::query_as!(CommittedDestination, "SELECT * FROM destination WHERE uid = ?", uid)
                .fetch_one(&self.connection)
                .await
                .map_err(|err| match err {
                    sqlx::Error::RowNotFound => error::DataPlaneError::NotFound { resource_id: uid.to_string() },
                    _ => {
                        let dp_err = error::DataPlaneError::from(err);
                        tracing::error!(
                            err = %dp_err,
                            destination_uid = uid,
                            "Failed to get destination",
                        );
                        dp_err
                    },
                })
        } else {
            sqlx::query_as!(CommittedDestination, "SELECT * FROM destination WHERE is_default_fallback = TRUE",)
                .fetch_one(&self.connection)
                .await
                .map_err(|err| match err {
                    sqlx::Error::RowNotFound => error::DataPlaneError::NotFound { resource_id: String::new() },
                    _ => {
                        let dp_err = error::DataPlaneError::from(err);
                        tracing::error!(
                            err = %dp_err,
                            "Failed to get default fallback shortcut",
                        );
                        dp_err
                    },
                })
        }
    }

    /// Add destination for shortcut.
    async fn add_destination(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        uid: &str,
        validated_url: &str,
        num_params: i64,
        is_fallback: bool,
        description: &str,
    ) -> error::DataPlaneResult<()> {
        sqlx::query("INSERT INTO destination (uid, url, num_params, is_fallback, is_default_fallback, description) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(uid)
            .bind(validated_url)
            .bind(num_params)
            .bind(is_fallback)
            .bind(false)
            .bind(description)
            .execute(&mut **tx)
            .await
            .map_err(|err| match err {
                sqlx::Error::Database(ref db_err) => {
                    if db_err.downcast_ref::<sqlx::sqlite::SqliteError>().is_unique_violation() {
                        error::DataPlaneError::AlreadyExists { resource_id: validated_url.to_string() }
                    } else {
                        let dp_err = error::DataPlaneError::from(err);
                        tracing::error!(
                            err = %dp_err,
                            destination_uid = uid,
                            url = validated_url,
                            "Failed to create destination (database error)",
                        );
                        dp_err
                    }
                },
                _ => {
                    let dp_err = error::DataPlaneError::from(err);
                    tracing::error!(
                        err = %dp_err,
                        destination_uid = uid,
                        url = validated_url,
                        "Failed to create destination",
                    );
                    dp_err
                },
            })?;
        Ok(())
    }

    /// Add one or more aliases for shortcut.
    ///
    /// Helper method that does not allocate Strings on return.
    async fn add_aliases<'a>(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        uid: &str,
        aliases: &'a [&str],
    ) -> error::DataPlaneResult<()> {
        for alias in aliases.iter() {
            Self::add_alias(tx, uid, alias).await?;
            tracing::info!(destination_uid = uid, alias, "Added alias for shortcut",);
        }
        Ok(())
    }

    /// Get all aliases for list of destinations.
    ///
    /// Helper method for [`list_shortcuts`] that builds an IN filter for the WHERE clause.
    /// SQLx does not support a generic form of binding a `Vec<T>` to a list in a SQL query (like
    /// with IN).
    async fn get_aliases_for_destinations(
        &self,
        destinations: &[CommittedDestination],
    ) -> error::DataPlaneResult<Vec<CommittedAlias>> {
        let mut query_builder = sqlx::query_builder::QueryBuilder::new(
            "SELECT destination_uid, NULL as \"uid?: String\", name FROM alias WHERE destination_uid IN (",
        );
        let mut filter_builder = query_builder.separated(",");
        destinations.iter().for_each(|destination| {
            filter_builder.push_bind(destination.uid.as_str());
        });
        filter_builder.push_unseparated(") ORDER BY destination_uid");
        query_builder
            .build()
            .map(|row| CommittedAlias {
                destination_uid: Some(row.get::<String, &str>("destination_uid")),
                uid: None,
                name: row.get::<String, &str>("name"),
            })
            .fetch_all(&self.connection)
            .await
            .map_err(error::DataPlaneError::from)
    }
}

#[tonic::async_trait]
impl HareDataPlaneClient for HareDataPlaneSqlite {
    async fn bootstrap(&self) -> error::DataPlaneResult<()> {
        sqlx::migrate!().run(&self.connection).await.inspect_err(|err| {
            tracing::error!(
                %err,
                "Failed to run migrations on database",
            );
        })?;
        tracing::info!("Ran migrations on database");
        Ok(())
    }

    async fn add_aliases_for_shortcut(
        &self,
        uid: &str,
        aliases: &[&str],
    ) -> error::DataPlaneResult<Option<Vec<CommittedAlias>>> {
        // Ensure UID corresponds to existing shortcut
        self.get_shortcut_by_uid(uid).await?;

        let mut tx = self.connection.begin().await?;
        self.add_aliases(&mut tx, uid, aliases).await?;
        tx.commit().await?;

        Ok(Some(
            aliases
                .iter()
                .map(|alias_name| CommittedAlias {
                    destination_uid: Some(uid.to_string()),
                    uid: None,
                    name: alias_name.to_string(),
                })
                .collect(),
        ))
    }

    async fn create_shortcut(
        &self,
        url: &str,
        is_fallback: bool,
        description: &str,
        aliases: &[&str],
    ) -> error::DataPlaneResult<CommittedShortcut> {
        if aliases.is_empty() {
            return Err(error::DataPlaneError::InvalidArgument {
                message: "Must provide at least one alias".to_string(),
            });
        }
        let validated_url = utils::validate_url(url).inspect_err(|err| {
            tracing::info!(
                %err,
                url,
                "Invalid URL for shortcut",
            );
        })?;
        let num_params = utils::gen_num_params_from_url(&validated_url);
        tracing::debug!(url = validated_url, "Number of params for URL is {}", num_params);
        let uid = uuid::Uuid::new_v4().to_string();

        let mut tx = self.connection.begin().await?;
        Self::add_destination(&mut tx, &uid, &validated_url, num_params, is_fallback, description).await?;
        tracing::info!(destination_uid = uid, "Created new destination");
        self.add_aliases(&mut tx, &uid, aliases).await?;
        tx.commit().await?;

        Ok(self.get_shortcut(Some(&uid), None, false).await?)
    }

    async fn delete_aliases_for_shortcut(
        &self,
        uid: &str,
        aliases: &[&str],
    ) -> error::DataPlaneResult<Option<Vec<CommittedAlias>>> {
        if aliases.is_empty() {
            return Ok(None);
        }

        let aliases_set = aliases.iter().collect::<std::collections::HashSet<_>>();
        let num_aliases = sqlx::query("SELECT COUNT(name) as num_aliases FROM alias WHERE destination_uid = ?")
            .bind(uid)
            .fetch_one(&self.connection)
            .await
            .inspect_err(|err| {
                tracing::error!(
                    %err,
                    destination_uid = uid,
                    "Failed to count number of aliases for destination",
                );
            })?
            .get::<u32, _>("num_aliases");
        if num_aliases == 0 {
            return Err(error::DataPlaneError::NotFound { resource_id: uid.to_string() });
        }
        if num_aliases <= aliases_set.len().try_into().unwrap() {
            return Err(error::DataPlaneError::FailedPrecondition {
                message: format!(
                    "Tried to delete {} aliases but only {} exist, and at least one alias must exist",
                    aliases_set.len(),
                    num_aliases,
                ),
            });
        }

        let mut tx = self.connection.begin().await?;
        let mut aliases_deleted = std::collections::HashSet::with_capacity(aliases_set.len());
        for alias in aliases_set.iter() {
            // NOTE: Deleting an alias that does not exist will not throw error in SQLite
            if Self::delete_alias(&mut tx, uid, alias).await? {
                tracing::info!(destination_uid = uid, alias, "Deleted alias for shortcut",);
                aliases_deleted.insert(alias);
            }
        }
        tx.commit().await?;
        tracing::info!(
            destination_uid = uid,
            "Deleted {}/{} aliases for shorcut",
            aliases_deleted.len(),
            aliases.len(),
        );

        Ok(Some(
            aliases_deleted
                .into_iter()
                .map(|alias_name| CommittedAlias {
                    destination_uid: Some(uid.to_string()),
                    uid: None,
                    name: alias_name.to_string(),
                })
                .collect::<Vec<_>>(),
        ))
    }

    async fn delete_shortcut(&self, uid: &str) -> error::DataPlaneResult<()> {
        sqlx::query("DELETE FROM destination WHERE uid = ?").bind(uid).execute(&self.connection).await.inspect_err(
            |err| {
                tracing::error!(
                    %err,
                    destination_uid = uid,
                    "Failed to delete shortcut",
                );
            },
        )?;
        tracing::info!(destination_uid = uid, "Deleted shortcut",);

        Ok(())
    }

    async fn get_default_fallback_shortcut(&self) -> error::DataPlaneResult<CommittedShortcut> {
        let destination = self.get_destination(None).await?;
        let aliases = self.get_aliases(&destination.uid).await?;
        Ok(CommittedShortcut { destination, aliases })
    }

    async fn set_default_fallback_shortcut(&self, uid: &str) -> error::DataPlaneResult<()> {
        let destination = self.get_destination(Some(uid)).await?;

        let mut tx = self.connection.begin().await?;
        // Unset current default fallback shortcut if exists
        match self.get_destination(None).await {
            Ok(default_fallback_destination) => {
                sqlx::query("UPDATE destination SET is_default_fallback = ? WHERE uid = ?")
                    .bind(false)
                    .bind(default_fallback_destination.uid)
                    .execute(&mut *tx)
                    .await
                    .inspect_err(|err| {
                        tracing::error!(
                            %err,
                            "Failed to unset default fallback shortcut",
                        );
                    })?;
            },
            Err(err) => match err {
                error::DataPlaneError::NotFound { resource_id: _ } => {},
                _ => {
                    tracing::error!(
                        %err,
                        "Failed to get current default fallback shortcut",
                    );
                    return Err(err);
                },
            },
        }
        // Set new default fallback shortcut
        sqlx::query("UPDATE destination SET is_default_fallback = ? WHERE uid = ?")
            .bind(true)
            .bind(&destination.uid)
            .execute(&mut *tx)
            .await
            .inspect_err(|err| {
                tracing::error!(
                    %err,
                    destination_uid = destination.uid,
                    "Failed to update default fallback shortcut",
                );
            })?;
        tx.commit().await?;

        Ok(())
    }

    async fn get_shortcut_by_uid(&self, uid: &str) -> error::DataPlaneResult<CommittedShortcut> {
        let destination = self.get_destination(Some(uid)).await?;
        let aliases = self.get_aliases(uid).await?;
        Ok(CommittedShortcut { destination, aliases })
    }

    async fn get_shortcut_by_alias(&self, alias: &str) -> error::DataPlaneResult<CommittedShortcut> {
        // Get all aliases for the destination that the provided alias corresponds to
        // Optimization to perform one query instead of two and using self.get_aliases
        let aliases: Vec<CommittedAlias> = sqlx::query_as!(
            CommittedAlias,
            "SELECT destination_uid, NULL as \"uid?: String\", name from alias where destination_uid = (SELECT destination_uid FROM alias where name = ?)",
            alias,
        )
            .fetch_all(&self.connection)
            .await
            .map_err(|err| {
                tracing::error!(
                    %err,
                    alias,
                    "Failed to get aliases for destination with alias",
                );
                err
            })?;

        if aliases.is_empty() {
            return Err(error::DataPlaneError::NotFound { resource_id: alias.to_string() });
        }

        let destination_uid = aliases[0].destination_uid.as_ref().unwrap();
        let destination = self.get_destination(Some(&destination_uid)).await?;

        Ok(CommittedShortcut { destination, aliases })
    }

    async fn list_shortcuts(
        &self,
        pagination_request: &PaginationRequest,
    ) -> error::DataPlaneResult<ListShortcutsResponse> {
        let continuation_token = if pagination_request.continuation_token.is_some() {
            ContinuationToken::try_from_str(&self.codec, pagination_request.continuation_token.as_ref().unwrap())
                .map_err(|err| match err {
                    error::DataPlaneError::SerdeJson(_) => {
                        tracing::info!(
                            %err,
                            continuation_token = pagination_request.continuation_token.as_ref().unwrap(),
                            "Invalid continuation token",
                        );
                        error::DataPlaneError::InvalidArgument { message: "Invalid continuation token".to_string() }
                    },
                    _ => {
                        tracing::warn!(
                            %err,
                            continuation_token = pagination_request.continuation_token.as_ref().unwrap(),
                            "Failed to process continuation token",
                        );
                        err
                    },
                })
        } else if let Some(page_size) = pagination_request.page_size {
            Ok(ContinuationToken::new(page_size as u32, 0))
        } else {
            Err(error::DataPlaneError::InvalidArgument {
                message: "Must supply either page_size or continuation_token".to_string(),
            })
        }?;
        tracing::debug!(
            "List shortcuts with page size {} and offset {}",
            continuation_token.page_size,
            continuation_token.offset,
        );

        let destinations: Vec<CommittedDestination> = sqlx::query_as!(
            CommittedDestination,
            "SELECT * FROM destination ORDER BY description, url LIMIT ? OFFSET ?",
            continuation_token.page_size,
            continuation_token.offset,
        )
        .fetch_all(&self.connection)
        .await
        .inspect_err(|err| {
            tracing::error!(
                %err,
                page_size = continuation_token.page_size,
                offset = continuation_token.offset,
                "Failed to get next page of destinations",
            );
        })?;
        let num_destinations = destinations.len();
        if num_destinations == 0 {
            return Ok(ListShortcutsResponse {
                shortcuts: CommittedShortcutList { shortcuts: Vec::new() },
                pagination: PaginationContinuation { next_continuation_token: None },
            });
        }

        // Each destination must have at least one alias, so the map size is guaranteed to have same capacity as destinations.
        let mut aliases_by_destination = self.get_aliases_for_destinations(&destinations).await.inspect_err(|err| {
            tracing::error!(
                %err,
                destination_uids = destinations.iter().map(|destination| destination.uid.as_str()).collect::<Vec<_>>().join(","),
                "Failed to get aliases for destinations",
            );
        })?.into_iter().fold(
            std::collections::HashMap::<String, Vec<CommittedAlias>>::with_capacity(num_destinations),
            |mut acc, alias| {
                let destination_uid = alias.destination_uid.as_ref().unwrap();
                if acc.contains_key(destination_uid) {
                    acc.get_mut(destination_uid.as_str()).unwrap().push(alias);
                } else {
                    // TODO: figure out how to avoid extra allocation here
                    // Allocating because key cannot be reference from value itself (I think)
                    acc.insert(alias.destination_uid.as_ref().unwrap().clone(), vec![alias]);
                }
                acc
            },
        );

        Ok(ListShortcutsResponse {
            shortcuts: CommittedShortcutList {
                shortcuts: destinations
                    .into_iter()
                    .map(|destination| {
                        let aliases = aliases_by_destination.remove(destination.uid.as_str()).unwrap();
                        CommittedShortcut { destination, aliases }
                    })
                    .collect(),
            },
            pagination: PaginationContinuation {
                next_continuation_token: Some(
                    ContinuationToken::new(
                        continuation_token.page_size,
                        continuation_token.offset + num_destinations as u32,
                    )
                    .try_to_string(&self.codec)?,
                ),
            },
        })
    }

    async fn update_shortcut(
        &self,
        uid: &str,
        url: Option<&str>,
        is_fallback: Option<bool>,
        description: Option<&str>,
    ) -> error::DataPlaneResult<CommittedShortcut> {
        let shortcut = self.get_shortcut_by_uid(uid).await?;
        // If nothing to change, just return the shortcut
        if !(url.is_some() || is_fallback.is_some() || description.is_some()) {
            return Ok(shortcut);
        }

        let update_url = url.unwrap_or(shortcut.destination.url.as_str());
        let update_is_fallback = is_fallback.unwrap_or(shortcut.destination.is_fallback);
        let update_description = description.unwrap_or(shortcut.destination.description.as_str());
        let destination: CommittedDestination = sqlx::query_as!(
            CommittedDestination,
            "UPDATE destination SET url = ?, is_fallback = ?, description = ? WHERE uid = ? RETURNING *",
            update_url,
            update_is_fallback,
            update_description,
            shortcut.destination.uid,
        )
        .fetch_one(&self.connection)
        .await
        .inspect_err(|err| {
            tracing::error!(
                %err,
                url,
                is_fallback,
                "Failed to update shortcut",
            );
        })?;
        tracing::info!(destination_uid = uid, "Updated shortcut");

        Ok(CommittedShortcut { destination, aliases: shortcut.aliases })
    }
}
