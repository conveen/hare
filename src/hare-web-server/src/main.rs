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

async fn get_listener() -> Result<tokio::net::TcpListener, std::io::Error> {
    let address = std::env::args().skip(1).take(1).next().unwrap_or("127.0.0.1:8001".to_string());
    tokio::net::TcpListener::bind(address).await
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    configure_logging();

    let listener = get_listener().await.unwrap_or_else(|err| {
        error!(%err, "Failed to bind to socket address");
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

    info!("Listening on {}", listener.local_addr().unwrap().to_string());
    axum::serve(
        listener,
        hare_web_server::app::create_app(data_plane_client)
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
                hare_web_server::audit_log::WebAuditRecordFactory {},
            )))
            .into_make_service_with_connect_info::<hare_common_utils::connect_info::ConnectionInformation>(),
    )
    .await
}
