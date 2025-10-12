use crate::{
    error::{self},
    model::{CommittedAlias, CommittedDestination, CommittedShortcut, CommittedShortcutList},
    pagination::ContinuationToken,
    utils, HareDataPlaneClient, ListShortcutsResponse,
};
use hare_common_model::pagination::{PaginationContinuation, PaginationRequest};
use sqlx::{error::DatabaseError, Row};

// Utility impl to convert from a database row to [`CommittedAlias`].
//
// Avoiding the use of [`sqlx::query_as`] makes development easier across RDBMS,
// and this utility avoids duplicate code for converting query results to the model types.
// If any of the column names are wrong get will panic.
impl<DB, R> From<R> for CommittedAlias
where
    DB: sqlx::Database,
    R: sqlx::Row<Database = DB>,
    for<'c> Option<String>: sqlx::Decode<'c, DB> + sqlx::Type<DB>,
    for<'c> String: sqlx::Decode<'c, DB> + sqlx::Type<DB>,
    for<'c> &'c str: sqlx::ColumnIndex<R>,
{
    fn from(row: R) -> Self {
        CommittedAlias {
            destination_uid: row.get::<Option<String>, _>("destination_uid"),
            uid: None,
            name: row.get::<String, _>("name"),
        }
    }
}

// Utility impl to convert from a database row to [`CommittedDestination`].
//
// Avoiding the use of [`sqlx::query_as`] makes development easier across RDBMS,
// and this utility avoids duplicate code for converting query results to the model types.
// If any of the column names are wrong get will panic.
impl<DB, R> From<R> for CommittedDestination
where
    DB: sqlx::Database,
    R: sqlx::Row<Database = DB>,
    for<'c> Option<String>: sqlx::Decode<'c, DB> + sqlx::Type<DB>,
    for<'c> String: sqlx::Decode<'c, DB> + sqlx::Type<DB>,
    for<'c> bool: sqlx::Decode<'c, DB> + sqlx::Type<DB>,
    for<'c> i32: sqlx::Decode<'c, DB> + sqlx::Type<DB>,
    for<'c> &'c str: sqlx::ColumnIndex<R>,
{
    fn from(row: R) -> Self {
        CommittedDestination {
            uid: row.get::<String, _>("uid"),
            url: row.get::<String, _>("url"),
            num_params: row.get::<i32, _>("num_params"),
            is_fallback: row.get::<bool, _>("is_fallback"),
            is_default_fallback: row.get::<bool, _>("is_default_fallback"),
            description: row.get::<String, _>("description"),
        }
    }
}

/// Utility trait for `rows_affected` function for query results.
///
/// SQLx doesn't have a trait for this, but each concrete query result type implements this function.
/// This trait fills that gap.
pub trait RowsAffected {
    fn rows_affected(&self) -> u64;
}

#[cfg(feature = "postgres")]
impl RowsAffected for sqlx::postgres::PgQueryResult {
    fn rows_affected(&self) -> u64 {
        self.rows_affected()
    }
}

#[cfg(feature = "sqlite")]
impl RowsAffected for sqlx::sqlite::SqliteQueryResult {
    fn rows_affected(&self) -> u64 {
        self.rows_affected()
    }
}

/// Associate an error type with a [`sqlx::Database`].
///
/// [`sqlx::Database`] doesn't have an associated error type even though each database engine has a dedicated error struct.
/// To make [`HareDataPlaneRelational`] generic over the engine _and_ error type without requiring [`std::marker::PhantomData`]
/// we use a custom trait to associate an error with each DB engine.
pub trait DatabaseWithAssociatedError: sqlx::Database {
    type Error: sqlx::error::DatabaseError;
}

#[cfg(feature = "postgres")]
impl DatabaseWithAssociatedError for sqlx::Postgres {
    type Error = sqlx::postgres::PgDatabaseError;
}

#[cfg(feature = "sqlite")]
impl DatabaseWithAssociatedError for sqlx::Sqlite {
    type Error = sqlx::sqlite::SqliteError;
}

/// Relational data plane client generic over the database engine.
pub struct HareDataPlaneRelational<DB: DatabaseWithAssociatedError> {
    codec: base64::engine::GeneralPurpose,
    connection: sqlx::Pool<DB>,
}

impl<DB: DatabaseWithAssociatedError> std::fmt::Debug for HareDataPlaneRelational<DB> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HareDataPlaneRelational").finish()
    }
}

impl<DB> HareDataPlaneRelational<DB>
where
    DB: DatabaseWithAssociatedError,
    <DB as sqlx::Database>::QueryResult: RowsAffected,
    for<'c> &'c mut <DB as sqlx::Database>::Connection: sqlx::Executor<'c, Database = DB>,
    for<'c> <DB as sqlx::Database>::Arguments<'c>: sqlx::IntoArguments<'c, DB>,
    for<'c> Option<String>: sqlx::Decode<'c, DB> + sqlx::Type<DB>,
    for<'c> String: sqlx::Decode<'c, DB> + sqlx::Encode<'c, DB> + sqlx::Type<DB>,
    for<'c> bool: sqlx::Decode<'c, DB> + sqlx::Type<DB> + sqlx::Encode<'c, DB>,
    for<'c> i32: sqlx::Decode<'c, DB> + sqlx::Type<DB> + sqlx::Encode<'c, DB>,
    for<'c> &'c str: sqlx::ColumnIndex<<DB as sqlx::Database>::Row> + sqlx::Encode<'c, DB> + sqlx::Type<DB>,
{
    pub fn from_connection(connection: sqlx::Pool<DB>) -> Self {
        let codec = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        HareDataPlaneRelational { codec, connection }
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
        tracing::debug!("Connected to database at URL: {}", database_url.as_ref());
        Ok(Self::from_connection(connection))
    }

    pub async fn try_from_env() -> error::DataPlaneResult<Self> {
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            return Self::try_from_url(database_url).await;
        }


        let driver = std::env::var("DATABASE_DRIVER")?;
        let user = std::env::var("DATABASE_USER")?;
        let password = std::env::var("DATABASE_PASSWORD")?;
        let host = std::env::var("DATABASE_HOST")?;
        let port = std::env::var("DATABASE_PORT")?;
        let name = std::env::var("DATABASE_NAME")?;
        let database_url = format!("{}://{}:{}@{}:{}/{}", driver, user, password, host, port, name);
        Self::try_from_url(database_url).await
    }

    /// Add one alias to a shortcut.
    async fn add_alias(tx: &mut sqlx::Transaction<'_, DB>, uid: &str, alias: &str) -> error::DataPlaneResult<()> {
        sqlx::query("INSERT INTO alias (destination_uid, name) VALUES ($1, $2)")
            .bind(uid)
            .bind(alias)
            .execute(&mut **tx)
            .await
            .map_err(|err| match err {
                sqlx::Error::Database(db_err)
                    if db_err
                        .downcast_ref::<<DB as DatabaseWithAssociatedError>::Error>()
                        .is_foreign_key_violation() =>
                {
                    error::DataPlaneError::NotFound { resource_id: uid.to_string() }
                },
                sqlx::Error::Database(db_err)
                    if db_err.downcast_ref::<<DB as DatabaseWithAssociatedError>::Error>().is_unique_violation() =>
                {
                    error::DataPlaneError::AlreadyExists { resource_id: alias.to_string() }
                },
                _ => {
                    let dp_err = error::DataPlaneError::from(err);
                    tracing::error!(
                        err = %dp_err,
                        destination_uid = uid,
                        "Unexpected database error when adding alias for shortcut",
                    );
                    dp_err
                },
            })
            .map(|_| ())
    }

    /// Delete one alias.
    async fn delete_alias(tx: &mut sqlx::Transaction<'_, DB>, uid: &str, alias: &str) -> error::DataPlaneResult<()> {
        let rows_affected = sqlx::query("DELETE FROM alias WHERE destination_uid = $1 AND name = $2")
            .bind(uid)
            .bind(alias)
            .execute(&mut **tx)
            .await
            .map_err(error::DataPlaneError::from)?
            .rows_affected();

        if rows_affected == 0 {
            return Err(error::DataPlaneError::NotFound { resource_id: alias.to_string() });
        }
        Ok(())
    }

    /// Get aliases for a destination.
    async fn get_aliases(&self, uid: &str) -> error::DataPlaneResult<Vec<CommittedAlias>> {
        Ok(sqlx::query("SELECT destination_uid, NULL as uid, name FROM alias WHERE destination_uid = $1")
            .bind(uid)
            .fetch_all(&self.connection)
            .await
            .map_err(error::DataPlaneError::from)
            .inspect_err(|err| {
                tracing::error!(
                    %err,
                    destination_uid = uid,
                    "Failed to get aliases for destination",
                );
            })?
            .into_iter()
            .map(CommittedAlias::from)
            .collect())
    }

    /// Get destination by UID
    async fn get_destination_by_uid(&self, uid: &str) -> error::DataPlaneResult<CommittedDestination> {
        sqlx::query("SELECT * FROM destination WHERE uid = $1")
            .bind(uid)
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
            .map(CommittedDestination::from)
    }

    /// Get the default fallback destination
    async fn get_default_fallback_destination(&self) -> error::DataPlaneResult<CommittedDestination> {
        sqlx::query("SELECT * FROM destination WHERE is_default_fallback = $1")
            .bind(true)
            .fetch_one(&self.connection)
            .await
            .map_err(|err| match err {
                sqlx::Error::RowNotFound => error::DataPlaneError::NotFound { resource_id: String::new() },
                _ => {
                    let dp_err = error::DataPlaneError::from(err);
                    tracing::error!(
                        err = %dp_err,
                        "Failed to get default fallback destination",
                    );
                    dp_err
                },
            })
            .map(CommittedDestination::from)
    }

    /// Get destination by UID or the default fallback.
    async fn get_destination(&self, maybe_uid: Option<&str>) -> error::DataPlaneResult<CommittedDestination> {
        match maybe_uid {
            Some(uid) => self.get_destination_by_uid(uid).await,
            None => self.get_default_fallback_destination().await,
        }
    }

    /// Add destination for shortcut.
    async fn add_destination(
        tx: &mut sqlx::Transaction<'_, DB>,
        uid: &str,
        validated_url: &str,
        num_params: i32,
        is_fallback: bool,
        description: &str,
    ) -> error::DataPlaneResult<()> {
        sqlx::query("INSERT INTO destination (uid, url, num_params, is_fallback, is_default_fallback, description) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(uid)
            .bind(validated_url)
            .bind(num_params)
            .bind(is_fallback)
            .bind(false)
            .bind(description)
            .execute(&mut **tx)
            .await
            .map_err(|err| match err {
                sqlx::Error::Database(db_err) if db_err.downcast_ref::<<DB as DatabaseWithAssociatedError>::Error>().is_unique_violation() => {
                    error::DataPlaneError::AlreadyExists { resource_id: validated_url.to_string() }
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
    async fn add_aliases(
        &self,
        tx: &mut sqlx::Transaction<'_, DB>,
        uid: &str,
        aliases: &[&str],
    ) -> error::DataPlaneResult<()> {
        for alias in aliases.iter() {
            Self::add_alias(tx, uid, alias).await?;
            tracing::info!(destination_uid = uid, alias, "Added alias for shortcut");
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
        if destinations.is_empty() {
            return Ok(Vec::new());
        }

        let query = format!(
            "SELECT destination_uid, NULL as \"uid?: String\", name FROM alias WHERE destination_uid IN ('{}') ORDER BY destination_uid",
            destinations.iter().map(|destination| destination.uid.as_str()).collect::<Vec<_>>().join("','"),
        );
        dbg!(&query);
        // let mut query_builder = sqlx::query_builder::QueryBuilder::new(
        //     "SELECT destination_uid, NULL as \"uid?: String\", name FROM alias WHERE destination_uid IN (",
        // );
        // let mut filter_builder = query_builder.separated(",");
        // destinations.iter().for_each(|destination| {
        //     filter_builder.push_bind(destination.uid.as_str());
        // });
        // filter_builder.push_unseparated(") ORDER BY destination_uid");
        // let query = query_builder.sql();

        sqlx::query(&query)
            .fetch_all(&self.connection)
            .await
            .map(|aliases| aliases.into_iter().map(CommittedAlias::from).collect())
            .map_err(error::DataPlaneError::from)
    }
}

#[tonic::async_trait]
impl<DB> HareDataPlaneClient for HareDataPlaneRelational<DB>
where
    DB: DatabaseWithAssociatedError,
    <DB as sqlx::Database>::QueryResult: RowsAffected,
    <DB as sqlx::Database>::Connection: sqlx::migrate::Migrate,
    for<'c> <DB as sqlx::Database>::Arguments<'c>: sqlx::IntoArguments<'c, DB>,
    for<'c> &'c mut DB::Connection: sqlx::Executor<'c, Database = DB>,
    for<'c> Option<String>: sqlx::Decode<'c, DB> + sqlx::Type<DB>,
    for<'c> String: sqlx::Decode<'c, DB> + sqlx::Encode<'c, DB> + sqlx::Type<DB>,
    for<'c> bool: sqlx::Decode<'c, DB> + sqlx::Encode<'c, DB> + sqlx::Type<DB>,
    for<'c> i32: sqlx::Decode<'c, DB> + sqlx::Encode<'c, DB> + sqlx::Type<DB>,
    for<'c> i64: sqlx::Decode<'c, DB> + sqlx::Type<DB>,
    for<'c> &'c str: sqlx::ColumnIndex<<DB as sqlx::Database>::Row> + sqlx::Encode<'c, DB> + sqlx::Type<DB>,
{
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

    async fn check_health(&self) -> error::DataPlaneResult<()> {
        let uid = uuid::Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO health_check (uid, check_field) VALUES ($1, $2)")
            .bind(uid.as_str())
            .bind(true)
            .execute(&self.connection)
            .await
            .map_err(error::DataPlaneError::from)
            .inspect_err(|err| {
                tracing::error!(check_uid = &uid, %err, "Failed to check health of database");
            })?;
        sqlx::query("DELETE FROM health_check WHERE uid = $1")
            .bind(uid.as_str())
            .execute(&self.connection)
            .await
            .map_err(error::DataPlaneError::from)
            .inspect_err(|err| {
                tracing::warn!(check_uid = &uid, %err, "Failed to remove health check record from database");
            })
            .ok();
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
        self.add_aliases(&mut tx, &uid, aliases).await?;
        tx.commit().await?;
        tracing::info!(destination_uid = uid, "Created new shortcut");

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

        let num_aliases = sqlx::query("SELECT COUNT(name) as num_aliases FROM alias WHERE destination_uid = $1")
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
            .get::<i64, _>("num_aliases");
        if num_aliases == 0 {
            return Err(error::DataPlaneError::NotFound { resource_id: uid.to_string() });
        }
        if num_aliases <= aliases.len().try_into().unwrap() {
            return Err(error::DataPlaneError::FailedPrecondition {
                message: format!(
                    "Tried to delete {} aliases but only {} exist, and at least one alias must exist",
                    aliases.len(),
                    num_aliases,
                ),
            });
        }

        let mut tx = self.connection.begin().await?;
        for alias in aliases.iter() {
            Self::delete_alias(&mut tx, uid, alias).await?;
            tracing::info!(destination_uid = uid, alias, "Deleted alias for shortcut");
        }
        tx.commit().await?;
        tracing::info!(destination_uid = uid, "Deleted {} aliases for shortcut", aliases.len(),);

        Ok(Some(
            aliases
                .iter()
                .map(|alias_name| CommittedAlias {
                    destination_uid: Some(uid.to_string()),
                    uid: None,
                    name: alias_name.to_string(),
                })
                .collect::<Vec<_>>(),
        ))
    }

    async fn delete_shortcut(&self, uid: &str) -> error::DataPlaneResult<()> {
        let result = sqlx::query("DELETE FROM destination WHERE uid = $1")
            .bind(uid)
            .execute(&self.connection)
            .await
            .inspect_err(|err| {
                tracing::error!(
                    %err,
                    destination_uid = uid,
                    "Failed to delete shortcut",
                );
            })?;

        if result.rows_affected() != 1 {
            return Err(error::DataPlaneError::NotFound { resource_id: uid.to_string() });
        }
        tracing::info!(destination_uid = uid, "Deleted shortcut");

        Ok(())
    }

    async fn get_default_fallback_shortcut(&self) -> error::DataPlaneResult<CommittedShortcut> {
        let destination = self.get_destination(None).await?;
        let aliases = self.get_aliases(&destination.uid).await?;
        Ok(CommittedShortcut { destination, aliases })
    }

    async fn set_default_fallback_shortcut(&self, uid: &str) -> error::DataPlaneResult<()> {
        let destination = self.get_destination(Some(uid)).await?;
        if !destination.is_fallback {
            return Err(error::DataPlaneError::InvalidArgument {
                message: "Shortcut must be a fallback to make it the default fallback".to_string(),
            });
        }

        let mut tx = self.connection.begin().await?;
        // Unset current default fallback shortcut if exists
        match self.get_destination(None).await {
            Ok(default_fallback_destination) => {
                sqlx::query("UPDATE destination SET is_default_fallback = $1 WHERE uid = $2")
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
        sqlx::query("UPDATE destination SET is_default_fallback = $1 WHERE uid = $2")
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
        let aliases: Vec<_> = sqlx::query("SELECT destination_uid, NULL as \"uid?: String\", name from alias where destination_uid = (SELECT destination_uid FROM alias where name = $1)")
            .bind(alias)
            .fetch_all(&self.connection)
            .await
            .map_err(|err| {
                tracing::error!(
                    %err,
                    alias,
                    "Failed to get aliases for destination with alias",
                );
                err
            })
            .map(|aliases| aliases.into_iter().map(CommittedAlias::from).collect())?;

        if aliases.is_empty() {
            return Err(error::DataPlaneError::NotFound { resource_id: alias.to_string() });
        }

        let destination_uid = aliases[0].destination_uid.as_ref().unwrap();
        let destination = self.get_destination(Some(destination_uid)).await?;

        Ok(CommittedShortcut { destination, aliases })
    }

    async fn list_shortcuts(
        &self,
        pagination_request: &PaginationRequest,
    ) -> error::DataPlaneResult<ListShortcutsResponse> {
        let continuation_token = ContinuationToken::<u32>::try_from_pagination_request(&self.codec, pagination_request)
            .map_err(|err| match err {
                error::DataPlaneError::SerdeJson(_) => {
                    tracing::info!(
                        %err,
                        continuation_token = pagination_request.continuation_token.as_ref(),
                        "Invalid continuation token",
                    );
                    error::DataPlaneError::InvalidArgument { message: "Invalid continuation token".to_string() }
                },
                _ => {
                    tracing::warn!(
                        %err,
                        continuation_token = pagination_request.continuation_token.as_ref(),
                        "Failed to process continuation token",
                    );
                    err
                },
            })?;

        tracing::debug!(
            "List shortcuts with page size {} and offset {}",
            continuation_token.page_size,
            continuation_token.offset,
        );

        let destinations: Vec<_> =
            sqlx::query("SELECT * FROM destination ORDER BY description, url LIMIT $1 OFFSET $2")
                // NOTE: Does casting u32 to i32 require an explicit bounds check?
                .bind(continuation_token.page_size as i32)
                .bind(continuation_token.offset as i32)
                .fetch_all(&self.connection)
                .await
                .inspect_err(|err| {
                    tracing::error!(
                        %err,
                        page_size = continuation_token.page_size,
                        offset = continuation_token.offset,
                        "Failed to get next page of destinations",
                    );
                })
                .map(|destinations| destinations.into_iter().map(CommittedDestination::from).collect())?;
        let num_destinations = destinations.len();
        if num_destinations == 0 {
            return Ok(ListShortcutsResponse {
                shortcuts: CommittedShortcutList { shortcuts: Vec::new() },
                pagination: PaginationContinuation { next_continuation_token: None },
            });
        }

        // Each destination must have at least one alias, so the map size is guaranteed to have same capacity as destinations.
        let mut aliases_by_destination = self.get_aliases_for_destinations(&destinations).await
            .inspect_err(|err| {
                tracing::error!(
                    %err,
                    destination_uids = destinations.iter().map(|destination| destination.uid.as_str()).collect::<Vec<_>>().join(","),
                    "Failed to get aliases for destinations",
                );
            })?
            .into_iter()
            .fold(
                std::collections::HashMap::<String, Vec<CommittedAlias>>::with_capacity(num_destinations),
                |mut acc, alias| {
                    let destination_uid = alias.destination_uid.as_ref().unwrap();
                    acc.entry(destination_uid.clone()).or_default().push(alias);
                    acc
                },
            );

        Ok(ListShortcutsResponse {
            shortcuts: CommittedShortcutList {
                shortcuts: destinations
                    .into_iter()
                    .map(|destination| {
                        let aliases = aliases_by_destination.remove(destination.uid.as_str()).unwrap_or_default();
                        CommittedShortcut { destination, aliases }
                    })
                    .collect(),
            },
            pagination: PaginationContinuation {
                next_continuation_token: Some(
                    ContinuationToken::new(
                        continuation_token.page_size,
                        Some(continuation_token.offset + num_destinations as u32),
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
        if is_fallback.is_some() && !is_fallback.as_ref().unwrap() && shortcut.destination.is_default_fallback {
            return Err(error::DataPlaneError::InvalidArgument {
                message: "Cannot set the is_fallback flag to false for the default fallback shortcut".to_string(),
            });
        }
        if !(url.is_some() || is_fallback.is_some() || description.is_some()) {
            return Ok(shortcut);
        }

        let update_url = url.unwrap_or(shortcut.destination.url.as_str());
        let update_is_fallback = is_fallback.unwrap_or(shortcut.destination.is_fallback);
        let update_description = description.unwrap_or(shortcut.destination.description.as_str());

        let destination = sqlx::query(
            "UPDATE destination SET url = $1, is_fallback = $2, description = $3 WHERE uid = $4 RETURNING *",
        )
        .bind(update_url)
        .bind(update_is_fallback)
        .bind(update_description)
        .bind(&shortcut.destination.uid)
        .fetch_one(&self.connection)
        .await
        .map_err(|err| match err {
            sqlx::Error::Database(db_err)
                if db_err.downcast_ref::<<DB as DatabaseWithAssociatedError>::Error>().is_unique_violation() =>
            {
                error::DataPlaneError::AlreadyExists { resource_id: update_url.to_string() }
            },
            _ => {
                let dp_err = error::DataPlaneError::from(err);
                tracing::error!(
                    err = %dp_err,
                    destination_uid = uid,
                    "Unexpected error when updating destination",
                );
                dp_err
            },
        })
        .map(CommittedDestination::from)?;
        tracing::info!(destination_uid = uid, "Updated shortcut");

        Ok(CommittedShortcut { destination, aliases: shortcut.aliases })
    }
}

/// Data plane client backed by Postgres.
#[cfg(feature = "postgres")]
pub type HareDataPlanePostgres = HareDataPlaneRelational<sqlx::Postgres>;

/// Data plane client backed by SQLite.
#[cfg(feature = "sqlite")]
pub type HareDataPlaneSqlite = HareDataPlaneRelational<sqlx::Sqlite>;
