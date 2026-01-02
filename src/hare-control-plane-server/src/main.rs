use hare_control_plane_model::server::HareControlPlaneServer;
use hare_control_plane_service::service::HareControlPlaneService;
use hare_data_plane_client::HareDataPlaneClient;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn configure_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap_or_else(|err| {
            eprintln!("Failed to initialize tracing subscriber: {}", err);
            std::process::exit(1);
        });
}

fn get_socket_address() -> Result<std::net::SocketAddr, std::net::AddrParseError> {
    std::env::args().skip(1).take(1).next().unwrap_or("127.0.0.1:5001".to_string()).as_str().parse()
}

#[tokio::main]
async fn main() -> Result<(), tonic::transport::Error> {
    configure_logging();

    let address = get_socket_address().unwrap_or_else(|err| {
        error!(%err, "Failed to parse socket address");
        std::process::exit(1);
    });

    #[cfg(feature = "postgres")]
    let data_plane_client = hare_data_plane_client::postgres::HareDataPlanePostgres::try_from_env().await;
    #[cfg(feature = "sqlite")]
    let data_plane_client = hare_data_plane_client::sqlite::HareDataPlaneSqlite::try_from_env().await;

    let data_plane_client = data_plane_client.unwrap_or_else(|err| {
        error!(%err, "Failed to create data plane client");
        std::process::exit(1);
    });
    data_plane_client.bootstrap().await.unwrap_or_else(|err| {
        error!(%err, "Failed to bootstrap database");
        std::process::exit(1);
    });

    let hare_control_plane = HareControlPlaneService::new(data_plane_client);
    info!("Listening on {}", address.to_string());
    tonic::transport::Server::builder()
        // Must come before SetRequestId middleware to ensure clients cannot inject request IDs
        .layer(hare_common_utils::header::FilterHeadersLayer::new(std::sync::Arc::new(vec![
            hare_common_utils::request_id::REQUEST_ID_HEADER_NAME.to_string(),
        ])))
        // Assign request ID to each request
        .layer(tower_http::request_id::SetRequestIdLayer::new(
            hare_common_utils::request_id::REQUEST_ID_HEADER_NAME.parse().unwrap(),
            hare_common_utils::request_id::RequestIdGenerator::default(),
        ))
        // Propagate request ID header to response
        .layer(tower_http::request_id::PropagateRequestIdLayer::new(
            hare_common_utils::request_id::REQUEST_ID_HEADER_NAME.parse().unwrap(),
        ))
        // Generate audit records for each request-reply pair
        .layer(hare_common_utils::audit_log::LogAuditRecordsLayer::new(std::sync::Arc::new(
            hare_control_plane_server::audit_log::ControlPlaneAuditRecordFactory {},
        )))
        .add_service(HareControlPlaneServer::new(hare_control_plane))
        .serve(address)
        .await
}
