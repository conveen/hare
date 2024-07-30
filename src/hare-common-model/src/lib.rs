pub use common::pagination;

pub mod common {
    pub mod pagination {
        tonic::include_proto!("app.conveen.hare.common.pagination");
    }
}
