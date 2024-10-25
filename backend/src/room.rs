use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_broadcast::{broadcast, InactiveReceiver, Receiver, Sender};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, trace};

use crate::status::Status;

#[derive(Default, Clone)]
pub struct Rooms {
    inner: Arc<Mutex<RoomsInner>>,
}

#[derive(Default)]
struct RoomsInner {
    map: HashMap<String, Arc<Mutex<Room>>>,
    stats: Stats,
}

impl Rooms {
    pub async fn stats(&self) -> Stats {
        self.inner.lock().await.stats.clone()
    }

    pub async fn get(&self, name: String) -> Arc<Mutex<Room>> {
        let mut inner = self.inner.lock().await;
        inner
            .map
            .entry(name)
            .or_insert_with(|| Arc::new(Mutex::new(Room::new())))
            .clone()
    }

    /// Deletes expired rooms and updates the statistics.
    pub async fn update(&self) -> Stats {
        let mut inner = self.inner.lock().await;
        let mut stats = Stats::default();

        let mut keys_to_keep = HashSet::new();
        for (key, room) in inner.map.iter() {
            let room = room.lock().await;
            stats.clients += room.num_clients();
            if room.is_expired() {
                stats.expired_rooms += 1;
            } else {
                stats.active_rooms += 1;
                keys_to_keep.insert(key.clone());
            }
        }
        inner.map.retain(|key, _| keys_to_keep.contains(key));
        inner.stats = stats.clone();
        stats
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub active_rooms: usize,
    pub expired_rooms: usize,
    pub clients: usize,
}

pub struct Room {
    sender: Sender<Status>,
    receiver: InactiveReceiver<Status>,
    last_status: Status,
    updated: Instant,
}

impl Room {
    pub fn new() -> Room {
        let (mut sender, receiver) = broadcast(1);
        sender.set_await_active(false);
        sender.set_overflow(true);
        Room {
            sender,
            receiver: receiver.deactivate(),
            last_status: Status::default(),
            updated: Instant::now(),
        }
    }

    pub fn num_clients(&self) -> usize {
        self.sender.receiver_count()
    }

    pub fn is_expired(&self) -> bool {
        self.num_clients() == 0 && self.updated.elapsed() > ROOM_DURATION
    }

    pub async fn write(&mut self, text: impl Into<Arc<str>>) -> Status {
        let mut text: Arc<str> = text.into();
        if text.len() > TEXT_MAX_LEN {
            debug!("Text too long ({} bytes), truncating.", text.len());
            // Try to trim the text at various indices
            for index in (0..=TEXT_MAX_LEN).rev() {
                if let Some(substr) = text.get(..index) {
                    text = Arc::from(substr);
                    break;
                }
            }
            assert!(text.len() <= TEXT_MAX_LEN);
        }

        self.last_status.text = text;
        self.update().await
    }

    pub async fn update(&mut self) -> Status {
        self.last_status.sequence = self.last_status.sequence.wrapping_add(1);
        self.last_status.clients = u32::try_from(self.num_clients()).unwrap_or(u32::MAX);
        trace!("Broadcasting status seq {}.", self.last_status.sequence);
        // ignore errors due to nobody listening
        let _ = self.sender.broadcast_direct(self.last_status.clone()).await;
        trace!("Done broadcasting.");
        self.updated = Instant::now();
        self.last_status.clone()
    }

    pub async fn subscribe(&mut self) -> (Receiver<Status>, Status) {
        trace!("Cloning receiver.");
        let receiver = self.receiver.activate_cloned();
        assert!(!receiver.await_active());
        assert!(receiver.overflow());
        (receiver, self.update().await)
    }
}

const ROOM_DURATION: Duration = Duration::from_secs(10 * 60); // 10 minutes
const TEXT_MAX_LEN: usize = 8_000; // 8 kB
