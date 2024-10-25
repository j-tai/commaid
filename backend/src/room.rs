use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_broadcast::{broadcast, InactiveReceiver, Receiver, Sender};
use tokio::sync::Mutex;
use tracing::{debug, trace};

use crate::status::Status;

#[derive(Default, Clone)]
pub struct Rooms {
    inner: Arc<Mutex<HashMap<String, Arc<Mutex<Room>>>>>,
}

impl Rooms {
    pub async fn get(&self, name: String) -> Arc<Mutex<Room>> {
        let mut inner = self.inner.lock().await;
        inner
            .entry(name)
            .or_insert_with(|| Arc::new(Mutex::new(Room::new())))
            .clone()
    }

    pub async fn cleanup(&self) -> usize {
        let mut inner = self.inner.lock().await;
        let mut keys_to_keep = HashSet::new();
        for (key, room) in inner.iter() {
            if !room.lock().await.is_expired() {
                keys_to_keep.insert(key.clone());
            }
        }
        let remove_count = inner.len() - keys_to_keep.len();
        inner.retain(|key, _| keys_to_keep.contains(key));
        remove_count
    }
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

    pub fn is_expired(&self) -> bool {
        self.sender.receiver_count() == 0 || self.updated.elapsed() > ROOM_DURATION
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
        self.last_status.clients = u32::try_from(self.sender.receiver_count()).unwrap_or(u32::MAX);
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
