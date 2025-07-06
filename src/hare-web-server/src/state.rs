use hare_data_plane_client::HareDataPlaneClient;

#[derive(Debug)]
pub struct AppState<D>
where
    D: HareDataPlaneClient + Send + Sync + 'static + std::fmt::Debug,
{
    pub data_plane_client: D,
}
