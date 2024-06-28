pub mod app {
    pub mod conveen {
        pub mod hare {
            use hare_common_model::app::conveen::hare::common;
            pub mod control_plane {
                tonic::include_proto!("app.conveen.hare.control_plane");
            }
        }
    }
}
