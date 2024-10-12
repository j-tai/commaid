use std::net::SocketAddr;
use std::path::Path;

use axum::routing::get_service;
use axum::Router;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use tracing::{debug, info, trace};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Set up logging with tracing
    const DEFAULT_FILTER: &str = if cfg!(debug_assertions) {
        concat!(env!("CARGO_CRATE_NAME"), "=trace,info")
    } else {
        concat!(env!("CARGO_CRATE_NAME"), "=info,warn")
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("COMMAID_LOG")
                .map(EnvFilter::new)
                .unwrap_or_else(|_| EnvFilter::try_new(DEFAULT_FILTER).unwrap()),
        )
        .with_writer(std::io::stderr)
        .init();
    trace!("Logging set up.");

    let index_path = Path::new(BUILD_PATH).join("index.html");
    let app_path = Path::new(BUILD_PATH).join("_app");
    let app = Router::new()
        .route("/", get_service(ServeFile::new(index_path)))
        .nest_service("/_app", ServeDir::new(app_path));

    let address: SocketAddr = "127.0.0.1:6033".parse().unwrap();
    info!("Starting server on {address}.");
    debug!("\n\n\thttp://localhost:{}\n", address.port());
    axum::serve(TcpListener::bind(address).await.unwrap(), app)
        .await
        .unwrap();
}

const BUILD_PATH: &str = if cfg!(debug_assertions) {
    "../frontend/build"
} else {
    "frontend/build"
};
