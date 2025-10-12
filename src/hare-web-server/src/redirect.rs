use std::sync::Arc;

use hare_data_plane_client::{
    error::{DataPlaneError, DataPlaneResult},
    HareDataPlaneClient,
};
use tracing::{error, info};

use crate::state::AppState;

/// Query for redirect.
///
/// A query consists of zero or one shorcut alias and zero or more parameters,
/// where the parameters are represented as space-separated in a single [`str`].
/// Without an alias the query is effectively a noop.
#[derive(Debug)]
pub(crate) struct Query<'a> {
    /// Raw value the alias and parameters are parsed from.
    raw: Option<&'a str>,
    /// Shortcut alias.
    pub alias: Option<&'a str>,
    /// Parameters for the alias.
    pub parameters: Option<&'a str>,
}

impl<'a> Query<'a> {
    /// Get the underlying raw query value.
    pub fn as_raw_ref(&self) -> Option<&'a str> {
        self.raw
    }

    /// Whether the query is empty.
    ///
    /// An empty query contains no alias or parameters.
    pub fn is_empty(&self) -> bool {
        self.raw.is_none() || self.raw.as_ref().unwrap().is_empty()
    }
}

impl<'a> From<Option<&'a str>> for Query<'a> {
    fn from(maybe_raw: Option<&'a str>) -> Self {
        if maybe_raw.is_none() || maybe_raw.as_ref().unwrap().is_empty() {
            return Self { raw: maybe_raw, alias: None, parameters: None };
        }

        let mut parts = maybe_raw.as_ref().unwrap().splitn(2, ' ');
        let alias = parts.next();
        let parameters = parts.next();
        Self { raw: maybe_raw, alias, parameters }
    }
}

/// Resolve query to shortcut and redirect to shortcut URL.
///
/// If the query is not empty and the alias resolves to an existing shortcut,
/// then returns the URL for that shortcut with the parameters injected.
/// Otherwise, if the alias is not provided or does not resolve to a shortcut,
/// then returns the provided fallback or default fallback shortcut URL with the raw query value injected.
#[tracing::instrument(skip(data_plane_client))]
pub(crate) async fn redirect_for_shortcut<'a, D: std::fmt::Debug + HareDataPlaneClient + Send + Sync + 'static>(
    query: Query<'a>,
    fallback: Option<&'a str>,
    data_plane_client: &D,
    request_id: &'a str,
) -> DataPlaneResult<String> {
    if !query.is_empty() {
        let alias = query.alias.unwrap();
        let parameters = query.parameters.unwrap_or("");

        if let Ok(shortcut) = data_plane_client.get_shortcut_by_alias(alias).await {
            info!("Resolved alias {} to shortcut with URL {}", alias, shortcut.destination.url);
            let parameters: String = url::form_urlencoded::byte_serialize(parameters.as_bytes()).collect();
            return Ok(shortcut.destination.url.replacen("{}", parameters.as_str(), 1));
        } else {
            info!("No shortcut for alias {}", alias);
        }
    }

    let shortcut = match fallback {
        // Resolve fallback alias to shortcut
        Some(alias) => {
            match data_plane_client.get_shortcut_by_alias(alias).await {
                Ok(shortcut) => {
                    info!("Resolved fallback alias {} to shortcut with URL {}", alias, shortcut.destination.url);
                    shortcut
                },
                // If fallback alias doesn't resolve, use the default fallback
                Err(_) => data_plane_client.get_default_fallback_shortcut().await.inspect_err(|err| {
                    error!(%err, "Failed to get default fallback shortcut");
                })?,
            }
        },
        // Use the default fallback shortcut
        None => data_plane_client.get_default_fallback_shortcut().await.inspect_err(|err| {
            error!(%err, "Failed to get default fallback shortcut");
        })?,
    };
    let parameters: String =
        url::form_urlencoded::byte_serialize(query.as_raw_ref().unwrap_or("").as_bytes()).collect();
    Ok(shortcut.destination.url.replace("{}", parameters.as_str()))
}

/// URL parameters for [`get_handler`].
#[derive(Debug, serde::Deserialize)]
pub struct UrlParameters {
    query: Option<String>,
    fallback: Option<String>,
}

/// Handler for `GET` request.
#[tracing::instrument(skip(state))]
pub async fn get_handler<D: HareDataPlaneClient + Send + Sync + 'static + std::fmt::Debug>(
    axum::extract::State(state): axum::extract::State<Arc<AppState<D>>>,
    axum::extract::Query(url_parameters): axum::extract::Query<UrlParameters>,
    axum::extract::Extension(request_id): axum::extract::Extension<tower_http::request_id::RequestId>,
) -> axum::response::Result<axum::response::Redirect> {
    let url = redirect_for_shortcut(
        Query::from(url_parameters.query.as_deref()),
        url_parameters.fallback.as_deref(),
        &state.data_plane_client,
        request_id.header_value().to_str().unwrap(),
    )
    .await
    .map_err(|err| match err {
        DataPlaneError::NotFound { resource_id: _ } => axum::http::StatusCode::NOT_FOUND,
        _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    Ok(axum::response::Redirect::permanent(url.as_str()))
}
