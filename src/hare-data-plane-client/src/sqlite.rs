use std::fmt::Debug;

use crate::{
    error::{self},
    model::{CommittedAlias, CommittedDestination, CommittedShortcut, CommittedShortcutList},
    pagination::ContinuationToken,
    utils, HareDataPlaneClient, ListShortcutsResponse,
};
use hare_common_model::app::conveen::hare::common::pagination::{PaginationContinuation, PaginationRequest};
use sqlx::{error::DatabaseError, Row};

pub struct HareDataPlaneSqlite {
    codec: base64::engine::GeneralPurpose,
    connection: sqlx::Pool<sqlx::Sqlite>,
}

impl Debug for HareDataPlaneSqlite {
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
        let connection = sqlx::Pool::connect(database_url.as_ref()).await.map_err(|err| {
            tracing::error!(
                %err,
                database_url = database_url.as_ref(),
                "Failed to connect to database",
            );
            err
        })?;
        Ok(Self::from_connection(connection))
    }

    /// Add one alias to a shortcut.
    ///
    /// Helper method for [`add_aliases_for_shortcut`] that maps SQLx SQLite errors.
    async fn add_alias_for_shortcut(
        &self,
        uid: &str,
        alias: &str,
    ) -> error::DataPlaneResult<sqlx::sqlite::SqliteQueryResult> {
        sqlx::query("INSERT INTO alias (destination_uid, name) VALUES (?, ?)")
            .bind(uid)
            .bind(alias)
            .execute(&self.connection)
            .await
            .map_err(|err| match err {
                sqlx::Error::Database(ref db_err) => {
                    if db_err.downcast_ref::<sqlx::sqlite::SqliteError>().is_foreign_key_violation() {
                        error::DataPlaneError::NotFound { resource_id: uid.to_string() }
                    } else if db_err.downcast_ref::<sqlx::sqlite::SqliteError>().is_unique_violation() {
                        error::DataPlaneError::AlreadyExists { resource_id: alias.to_string() }
                    } else {
                        tracing::error!(
                            %err,
                            destination_uid = uid,
                            "Unexpected database error when adding alias for shortcut",
                        );
                        error::DataPlaneError::from(err)
                    }
                },
                _ => {
                    tracing::error!(
                        %err,
                        destination_uid = uid,
                        "Unexpected error when adding alias for shortcut",
                    );
                    error::DataPlaneError::from(err)
                },
            })
    }

    /// Add one or more aliases for shortcut.
    ///
    /// Helper method that does not allocate Strings on return.
    async fn _add_aliases_for_shortcut<'a>(
        &self,
        uid: &str,
        aliases: &'a [&str],
    ) -> error::DataPlaneResult<std::collections::HashSet<&'a str>> {
        let aliases_set = aliases.into_iter().map(|a| *a).collect::<std::collections::HashSet<_>>();
        for alias in &aliases_set {
            self.add_alias_for_shortcut(uid, alias).await?;
            tracing::info!(destination_uid = uid, alias, "Added alias for shortcut",);
        }
        Ok(aliases_set)
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
        sqlx::migrate!().run(&self.connection).await.map_err(|err| {
            tracing::error!(
                %err,
                "Failed to run migrations on database",
            );
            err
        })?;
        tracing::info!("Ran migrations on database");
        Ok(())
    }

    async fn add_aliases_for_shortcut(
        &self,
        uid: &str,
        aliases: &[&str],
    ) -> error::DataPlaneResult<Option<Vec<String>>> {
        let aliases_set = self._add_aliases_for_shortcut(uid, aliases).await?;
        Ok(Some(aliases_set.iter().map(|alias| alias.to_string()).collect()))
    }

    async fn create_shortcut(
        &self,
        url: &str,
        is_fallback: bool,
        is_default_fallback: bool,
        description: &str,
        aliases: &[&str],
    ) -> error::DataPlaneResult<String> {
        if aliases.is_empty() {
            return Err(error::DataPlaneError::InvalidArgument {
                message: "Must provide at least one alias".to_string(),
            });
        }

        let validated_url = utils::validate_url(url).map_err(|err| {
            tracing::info!(
                %err,
                url,
                "Invalid URL for shortcut",
            );
            err
        })?;
        let num_params = utils::gen_num_params_from_url(&validated_url);
        tracing::debug!(url = validated_url, "Number of params for URL is {}", num_params);
        let uid = uuid::Uuid::new_v4().to_string();

        sqlx::query("INSERT INTO destination (uid, url, num_params, is_fallback, is_default_fallback, description) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(&uid)
            .bind(&validated_url)
            .bind(num_params)
            .bind(is_fallback)
            .bind(is_default_fallback)
            .bind(description)
            .execute(&self.connection)
            .await
            .map_err(|err| match err {
                sqlx::Error::Database(ref db_err) => {
                    if db_err.downcast_ref::<sqlx::sqlite::SqliteError>().is_unique_violation() {
                        error::DataPlaneError::AlreadyExists { resource_id: validated_url.clone() }
                    } else {
                        tracing::error!(
                            %err,
                            destination_uid = uid,
                            url = validated_url,
                            "Failed to create destination (database error)",
                        );
                        error::DataPlaneError::from(err)
                    }
                },
                _ => {
                    tracing::error!(
                        %err,
                        destination_uid = uid,
                        url = validated_url,
                        "Failed to create destination",
                    );
                    error::DataPlaneError::from(err)
                },
            })?;
        tracing::info!(destination_uid = uid, "Created new destination",);

        self._add_aliases_for_shortcut(&uid, aliases).await?.len();

        Ok(uid)
    }

    async fn delete_aliases_for_shortcut(
        &self,
        uid: &str,
        aliases: &[&str],
    ) -> error::DataPlaneResult<Option<Vec<String>>> {
        if aliases.is_empty() {
            return Ok(None);
        }

        let aliases_set = aliases.iter().collect::<std::collections::HashSet<_>>();
        let num_aliases = sqlx::query("SELECT COUNT(name) as num_aliases FROM alias WHERE destination_uid = ?")
            .bind(uid)
            .fetch_one(&self.connection)
            .await
            .map_err(|err| {
                tracing::error!(
                    %err,
                    destination_uid = uid,
                    "Failed to count number of aliases for destination",
                );
                err
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

        let mut aliases_deleted = std::collections::HashSet::with_capacity(aliases_set.len());
        for alias in aliases_set.iter() {
            // NOTE: Deleting an alias that does not exist will not throw error in SQLite
            if sqlx::query("DELETE FROM alias WHERE destination_uid = ? AND name = ? RETURNING *")
                .bind(uid)
                .bind(alias)
                .execute(&self.connection)
                .await
                .map_err(|err| {
                    tracing::error!(
                        %err,
                        destination_uid = uid,
                        alias,
                        "Failed to delete alias for destination",
                    );
                    err
                })?
                .rows_affected()
                == 1
            {
                tracing::info!(destination_uid = uid, alias, "Deleted alias for shortcut",);
                aliases_deleted.insert(alias);
            }
        }
        tracing::info!(
            destination_uid = uid,
            "Deleted {}/{} aliases for shorcut",
            aliases_deleted.len(),
            aliases.len(),
        );

        Ok(Some(aliases_deleted.into_iter().map(|alias| alias.to_string()).collect::<Vec<_>>()))
    }

    async fn delete_shortcut(&self, uid: &str) -> error::DataPlaneResult<()> {
        sqlx::query("DELETE FROM destination WHERE uid = ?").bind(uid).execute(&self.connection).await.map_err(
            |err| {
                tracing::error!(
                    %err,
                    destination_uid = uid,
                    "Failed to delete shortcut",
                );
                err
            },
        )?;
        tracing::info!(destination_uid = uid, "Deleted shortcut",);

        Ok(())
    }

    async fn get_default_fallback_shortcut(&self) -> error::DataPlaneResult<CommittedShortcut> {
        let destination: CommittedDestination =
            sqlx::query_as!(CommittedDestination, "SELECT * FROM destination WHERE is_default_fallback = TRUE",)
                .fetch_one(&self.connection)
                .await
                .map_err(|err| match err {
                    sqlx::Error::RowNotFound => error::DataPlaneError::NotFound { resource_id: String::new() },
                    _ => {
                        tracing::error!(
                            %err,
                            "Failed to get default fallback shortcut",
                        );
                        error::DataPlaneError::from(err)
                    },
                })?;

        let aliases = sqlx::query_as!(
            CommittedAlias,
            "SELECT destination_uid, NULL AS \"uid?: String\", name FROM alias WHERE destination_uid = ?",
            destination.uid
        )
        .fetch_all(&self.connection)
        .await
        .map_err(|err| {
            tracing::error!(
                %err,
                destination_uid = destination.uid,
                "Failed to get aliases for default fallback shortcut",
            );
            err
        })?;
        Ok(CommittedShortcut { destination, aliases })
    }

    async fn get_shortcut_by_uid(&self, uid: &str) -> error::DataPlaneResult<CommittedShortcut> {
        let destination: CommittedDestination =
            sqlx::query_as!(CommittedDestination, "SELECT * FROM destination WHERE uid = ?", uid)
                .fetch_one(&self.connection)
                .await
                .map_err(|err| match err {
                    sqlx::Error::RowNotFound => error::DataPlaneError::NotFound { resource_id: uid.to_string() },
                    _ => {
                        tracing::error!(
                            %err,
                            destination_uid = uid,
                            "Failed to get destination",
                        );
                        error::DataPlaneError::from(err)
                    },
                })?;

        let aliases = sqlx::query_as!(
            CommittedAlias,
            "SELECT destination_uid, NULL AS \"uid?: String\", name FROM alias WHERE destination_uid = ?",
            uid
        )
        .fetch_all(&self.connection)
        .await
        .map_err(|err| {
            tracing::error!(
                %err,
                destination_uid = uid,
                "Failed to get aliases for destination",
            );
            err
        })?;
        Ok(CommittedShortcut { destination, aliases })
    }

    async fn get_shortcut_by_alias(&self, alias: &str) -> error::DataPlaneResult<CommittedShortcut> {
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
        let destination =
            sqlx::query_as!(CommittedDestination, "SELECT * FROM destination WHERE uid = ?", destination_uid)
                .fetch_one(&self.connection)
                .await
                .map_err(|err| {
                    tracing::error!(
                        %err,
                        destination_uid,
                        "Failed to get destination",
                    );
                    err
                })?;

        Ok(CommittedShortcut { destination, aliases })
    }

    async fn list_shortcuts(
        &self,
        pagination_request: &PaginationRequest,
    ) -> error::DataPlaneResult<ListShortcutsResponse> {
        let continuation_token = if pagination_request.continuation_token.is_some() {
            ContinuationToken::try_from_str(&self.codec, pagination_request.continuation_token.as_ref().unwrap())
                .map_err(|err| match err {
                    error::DataPlaneError::Serde(_) => {
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
        .map_err(|err| {
            tracing::error!(
                %err,
                page_size = continuation_token.page_size,
                offset = continuation_token.offset,
                "Failed to get next page of destinations",
            );
            err
        })?;
        let num_destinations = destinations.len();
        if num_destinations == 0 {
            return Ok(ListShortcutsResponse {
                shortcuts: CommittedShortcutList { shortcuts: Vec::new() },
                pagination: PaginationContinuation { next_continuation_token: None },
            });
        }

        // Each destination must have at least one alias, so the map size is guaranteed to have same capacity as destinations.
        let mut aliases_by_destination = self.get_aliases_for_destinations(&destinations).await.map_err(|err| {
            tracing::error!(
                %err,
                destination_uids = destinations.iter().map(|destination| destination.uid.as_str()).collect::<Vec<_>>().join(","),
                "Failed to get aliases for destinations",
            );
            err
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
        uid: Option<&str>,
        alias: Option<&str>,
        url: Option<&str>,
        is_fallback: Option<bool>,
        is_default_fallback: Option<bool>,
        description: Option<&str>,
    ) -> error::DataPlaneResult<Option<CommittedShortcut>> {
        let shortcut = if uid.is_some() {
            self.get_shortcut_by_uid(uid.unwrap()).await?
        } else if alias.is_some() {
            self.get_shortcut_by_alias(alias.unwrap()).await?
        } else {
            return Err(error::DataPlaneError::InvalidArgument {
                message: "Must supply one of shortcut uid or alias".to_string(),
            });
        };
        // If nothing to change, just return the shortcut
        if !(url.is_some() || is_fallback.is_some() || is_default_fallback.is_some() || description.is_some()) {
            return Ok(Some(shortcut));
        }

        let update_url = url.unwrap_or(shortcut.destination.url.as_str());
        let update_is_fallback = is_fallback.unwrap_or(shortcut.destination.is_fallback);
        let update_is_default_fallback = is_default_fallback.unwrap_or(shortcut.destination.is_default_fallback);
        let update_description = description.unwrap_or(shortcut.destination.description.as_str());
        let destination: CommittedDestination = sqlx::query_as!(
            CommittedDestination,
            "UPDATE destination SET url = ?, is_fallback = ?, is_default_fallback = ?, description = ? RETURNING *",
            update_url,
            update_is_fallback,
            update_is_default_fallback,
            update_description,
        )
        .fetch_one(&self.connection)
        .await
        .map_err(|err| {
            tracing::error!(
                %err,
                url,
                is_fallback,
                is_default_fallback,
                "Failed to update shortcut",
            );
            err
        })?;
        tracing::info!(destination_uid = uid, "Updated shortcut",);

        Ok(Some(CommittedShortcut { destination, aliases: shortcut.aliases }))
    }
}
