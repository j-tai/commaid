use std::time::{Duration, Instant};

use tracing::{debug, trace};

use crate::room::Rooms;

pub async fn cleanup(rooms: Rooms) -> ! {
    loop {
        tokio::time::sleep(CLEANUP_DELAY).await;
        trace!("Cleaning up rooms and collecting stats...");

        let start = Instant::now();
        let stats = rooms.update().await;
        let duration = start.elapsed();

        let seconds = duration.as_secs_f64();
        if stats.expired_rooms == 0 {
            trace!(?stats, "Cleanup took {seconds:.3}s.");
        } else {
            debug!(?stats, "Cleanup took {seconds:.3}s.");
        }
    }
}

const CLEANUP_DELAY: Duration = Duration::from_secs(60);
