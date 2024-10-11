use tracing::{debug, error, info};

use hare_common_utils::request_id::request_id_interceptor;
use hare_control_plane_model::server::HareControlPlaneServer;
use hare_control_plane_service::service::HareControlPlaneService;
use hare_data_plane_client::{sqlite::HareDataPlaneSqlite, HareDataPlaneClient};
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

async fn get_data_plane_client() -> Result<HareDataPlaneSqlite, Box<dyn std::error::Error>> {
    let data_plane_client_url = std::env::var("DATABASE_URL")?;
    let data_plane_client = HareDataPlaneSqlite::try_from_url(data_plane_client_url.as_str()).await?;
    debug!("Connected to database at URL: {}", &data_plane_client_url);
    Ok(data_plane_client)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    configure_logging();

    let address = get_socket_address().unwrap_or_else(|err| {
        error!(%err, "Failed to parse socket address");
        std::process::exit(1);
    });

    let data_plane_client = get_data_plane_client().await.unwrap_or_else(|err| {
        error!(%err, "Failed to create data plane client");
        std::process::exit(1);
    });
    data_plane_client.bootstrap().await.unwrap_or_else(|err| {
        error!(%err, "Failed to bootstrap database");
        std::process::exit(1);
    });

    let hare_control_plane = HareControlPlaneService::<HareDataPlaneSqlite>::new(data_plane_client);
    info!("Listening on {}", address.to_string());
    tonic::transport::Server::builder()
        .add_service(HareControlPlaneServer::with_interceptor(hare_control_plane, request_id_interceptor))
        .serve(address)
        .await?;
    Ok(())
}
