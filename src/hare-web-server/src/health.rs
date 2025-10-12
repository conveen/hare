use std::sync::Arc;

use hare_data_plane_client::HareDataPlaneClient;

use crate::state::AppState;

/// Health check modes.
#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthCheckMode {
    Full,
    Simple,
}

impl Default for HealthCheckMode {
    fn default() -> Self {
        Self::Simple
    }
}

/// URL parameters for [`get_handler`].
#[derive(Debug, Default, serde::Deserialize)]
pub struct UrlParameters {
    #[serde(default = "Default::default")]
    mode: HealthCheckMode,
}

#[tracing::instrument(skip(state))]
pub async fn get_handler<D: HareDataPlaneClient + Send + Sync + 'static + std::fmt::Debug>(
    axum::extract::State(state): axum::extract::State<Arc<AppState<D>>>,
    axum::extract::Query(url_parameters): axum::extract::Query<UrlParameters>,
    axum::extract::Extension(request_id): axum::extract::Extension<tower_http::request_id::RequestId>,
) -> axum::response::Result<axum::http::StatusCode> {
    if url_parameters.mode == HealthCheckMode::Full {
        if let Err(_) = &state.data_plane_client.check_health().await {
            return Err(axum::http::StatusCode::FAILED_DEPENDENCY.into());
        }
    }

    Ok(axum::http::StatusCode::OK)
}
