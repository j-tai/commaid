use std::future::pending;

use async_signal::{Signal, Signals};
use tokio_stream::StreamExt;
use tracing::{info, warn};

/// Returns a future which resolves when a shutdown signal is received.
pub async fn shutdown_signal() {
    let mut signals = match Signals::new([Signal::Term, Signal::Int]) {
        Ok(inner) => inner,
        Err(e) => {
            warn!("Failed to register signal handlers: {e}");
            pending::<()>().await;
            unreachable!();
        }
    };

    while let Some(signal) = signals.next().await {
        match signal {
            Ok(signal) => {
                info!("Received signal {signal:?}; shutting down...");
                return;
            }
            Err(e) => warn!("Error in signal handler: {e}"),
        }
    }
    unreachable!();
}
