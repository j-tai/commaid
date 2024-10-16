mod daemon;
mod handler;
mod room;
mod server;
mod status;

use std::net::SocketAddr;
use std::path::Path;

use axum::routing::{get, get_service};
use axum::Router;
use room::Rooms;
use tempfile::TempDir;
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

    // Unpack frontend files
    let temp_dir = TempDir::new().unwrap();
    debug!("Temporary directory is {:?}.", temp_dir.path());
    let build = unpack_frontend(temp_dir.path());

    // Start cleanup daemon
    let rooms = Rooms::default();
    let rooms2 = rooms.clone();
    tokio::spawn(daemon::cleanup(rooms2));

    let index_path = build.join("index.html");
    let app_path = build.join("_app");
    let app = Router::new()
        .route("/", get_service(ServeFile::new(&index_path)))
        .route("/connect", get(handler::handle))
        .nest_service("/_app", ServeDir::new(app_path))
        .with_state(rooms);

    let address: SocketAddr = "127.0.0.1:6033".parse().unwrap();
    info!("Starting server on {address}.");
    debug!("\n\n\thttp://localhost:{}\n", address.port());
    axum::serve(
        TcpListener::bind(address).await.unwrap(),
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(server::shutdown_signal())
    .await
    .unwrap();
}

fn unpack_frontend(dir: &Path) -> &Path {
    #[cfg(debug_assertions)]
    {
        let _ = dir;
        Path::new(BUILD_PATH)
    }
    #[cfg(not(debug_assertions))]
    {
        BUILD_DIR.extract(dir).unwrap();
        dir
    }
}

#[cfg(debug_assertions)]
const BUILD_PATH: &str = "../frontend/build";
#[cfg(not(debug_assertions))]
const BUILD_DIR: include_dir::Dir =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../frontend/build");
