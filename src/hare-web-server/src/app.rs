use hare_data_plane_client::HareDataPlaneClient;

pub fn create_app<D: HareDataPlaneClient + Send + Sync + 'static + std::fmt::Debug>(
    data_plane_client: D,
) -> axum::Router {
    axum::Router::new()
        .route("/", axum::routing::get(crate::redirect::get_handler))
        .with_state(std::sync::Arc::new(crate::state::AppState { data_plane_client }))
}
