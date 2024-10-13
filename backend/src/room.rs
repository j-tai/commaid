use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_broadcast::{broadcast, InactiveReceiver, Receiver, Sender};
use tokio::sync::Mutex;

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
    sender: Sender<Arc<str>>,
    receiver: InactiveReceiver<Arc<str>>,
    last_message: Arc<str>,
    updated: Instant,
}

impl Room {
    pub fn new() -> Room {
        let (mut sender, receiver) = broadcast(1);
        sender.set_overflow(true);
        Room {
            sender,
            receiver: receiver.deactivate(),
            last_message: "".into(),
            updated: Instant::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.sender.receiver_count() == 0 || self.updated.elapsed() > ROOM_DURATION
    }

    pub async fn write(&mut self, message: impl Into<Arc<str>>) {
        let message = message.into();
        self.sender.broadcast_direct(message.clone()).await.unwrap();
        self.last_message = message;
        self.updated = Instant::now();
    }

    pub fn subscribe(&self) -> (Receiver<Arc<str>>, Arc<str>) {
        (self.receiver.activate_cloned(), self.last_message.clone())
    }
}

const ROOM_DURATION: Duration = Duration::from_secs(60 * 60); // 10 minutes
