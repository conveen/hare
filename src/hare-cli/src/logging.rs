use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn configure_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .try_init()
        .unwrap_or_else(|err| {
            eprintln!("Failed to initialize tracing subscriber: {}", err);
            std::process::exit(1);
        });
}
