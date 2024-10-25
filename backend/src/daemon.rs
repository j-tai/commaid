use std::time::{Duration, Instant};

use tracing::{debug, trace};

use crate::room::Rooms;

pub async fn cleanup(rooms: Rooms) -> ! {
    loop {
        tokio::time::sleep(CLEANUP_DELAY).await;
        trace!("Cleaning up rooms...");
        let start = Instant::now();
        let affected = rooms.cleanup().await;
        let duration = start.elapsed();
        let seconds = duration.as_secs_f64();
        if affected == 0 {
            trace!("No rooms affected, took {seconds:.3}s.");
        } else {
            debug!("{affected} room(s) removed, took {seconds:.3}s.");
        }
    }
}

const CLEANUP_DELAY: Duration = Duration::from_secs(5 * 60);
