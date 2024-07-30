use hare_common_model::common;

pub use control_plane::*;

pub mod client {
    pub use super::control_plane::hare_control_plane_client::HareControlPlaneClient;
}

pub mod server {
    pub use super::control_plane::hare_control_plane_server::{HareControlPlane, HareControlPlaneServer};
}

mod control_plane {
    tonic::include_proto!("app.conveen.hare.control_plane");
}
