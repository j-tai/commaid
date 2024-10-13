use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use async_broadcast::RecvError;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use tracing::{debug, info, instrument, warn};

use crate::room::{Rooms, Status};

#[derive(Deserialize)]
pub struct HandleParams {
    room: String,
}

pub async fn handle(
    rooms: State<Rooms>,
    info: ConnectInfo<SocketAddr>,
    form: Query<HandleParams>,
    upgrade: WebSocketUpgrade,
) -> Result<impl IntoResponse, StatusCode> {
    let room_name = form.0.room;
    // Validate room name
    if !matches!(room_name.len(), 6..=16) || !room_name.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(upgrade.on_upgrade(move |socket| websocket(info.ip(), socket, rooms.0, room_name)))
}

#[instrument(skip_all, fields(ip = %ip, room = room_name))]
async fn websocket(ip: IpAddr, socket: WebSocket, rooms: Rooms, room_name: String) {
    info!("Client connected.");
    match try_websocket(socket, rooms, room_name).await {
        Ok(()) => info!("Client disconnected."),
        Err(error) => warn!(%error, "Client disconnected."),
    }
}

async fn try_websocket(
    mut socket: WebSocket,
    rooms: Rooms,
    room_name: String,
) -> Result<(), Box<dyn Error>> {
    let room = rooms.get(room_name).await;
    let (mut receiver, initial_status) = room.lock().await.subscribe();
    let client_id = CLIENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed);

    // What the client currently sees
    let mut current = initial_status;
    if !current.text.is_empty() {
        socket.send(Message::Text(current.text.to_string())).await?;
    }

    loop {
        tokio::select! {
            // Receive update
            text = receiver.recv_direct() => {
                let new_status = match text {
                    Ok(text) => text,
                    Err(RecvError::Overflowed(_)) => continue,
                    Err(e) => return Err(e.into()),
                };
                if current != new_status {
                    current = new_status;
                    // Don't send back the user's own changes
                    if current.updated_by != client_id {
                        socket.send(Message::Text(current.text.to_string())).await?;
                    }
                }
            }
            // User typed something
            message = socket.recv() => {
                let Some(message) = message else {
                    debug!("Client closed connection.");
                    return Ok(()); // client closed connection
                };
                if let Message::Text(text) = message? {
                    let new_status = Status { text: Arc::from(&text[..]), updated_by: client_id };
                    let mut room = room.lock().await;
                    current = new_status.clone();
                    room.write(new_status).await;
                }
            }
        }
    }
}

static CLIENT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
