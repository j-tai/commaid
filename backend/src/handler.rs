use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use async_broadcast::RecvError;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use tokio::sync::Mutex;
use tracing::{debug, info, instrument, trace, warn};

use crate::room::{Room, Rooms, Stats};
use crate::status::Status;

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
    if room_name.len() != 6 || !room_name.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(upgrade.on_upgrade(move |socket| websocket(info.ip(), socket, rooms.0, room_name)))
}

#[instrument(skip_all, fields(ip = %ip, room = room_name))]
async fn websocket(ip: IpAddr, socket: WebSocket, rooms: Rooms, room_name: String) {
    info!("Client connected.");
    let room = rooms.get(room_name).await;

    match try_websocket(socket, &room).await {
        Ok(()) => info!("Client disconnected."),
        Err(error) => warn!(%error, "Client disconnected."),
    }

    room.lock().await.update().await;
}

async fn try_websocket(
    mut socket: WebSocket,
    room: &Arc<Mutex<Room>>,
) -> Result<(), Box<dyn Error>> {
    let (mut receiver, initial_status) = room.lock().await.subscribe().await;

    // What the client currently sees
    let mut current = initial_status;
    if let Some(message) = current.diff(&Status::default()) {
        socket.send(Message::Text(message)).await?;
    }

    loop {
        trace!("Waiting for message.");
        let new_status = tokio::select! {
            // Received update
            result = receiver.recv_direct() => {
                trace!("Received update from room.");
                match result {
                    Ok(new_status) => new_status,
                    Err(RecvError::Overflowed(_)) => {
                        trace!("Overflowed, ignoring.");
                        continue;
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            // User typed something
            message = socket.recv() => {
                trace!("Received message from user.");
                let Some(message) = message else {
                    debug!("Client closed connection.");
                    return Ok(()); // client closed connection
                };
                let Message::Text(text) = message? else {
                    trace!("Ignoring non-text message.");
                    continue;
                };
                let text: Arc<str> = Arc::from(text);
                current.text = text.clone();
                trace!("Sending update to room.");
                room.lock().await.write(text).await
            }
            // Send keepalive at least every 30 seconds
            _ = tokio::time::sleep(Duration::from_secs(30)) => {
                trace!("Sending keepalive.");
                socket.send(Message::Text(String::new())).await?;
                continue;
            }
        };

        if new_status > current {
            trace!(
                "Updating status, seq {} -> {}.",
                current.sequence,
                new_status.sequence,
            );
            if let Some(message) = new_status.diff(&current) {
                trace!("Sending message to user.");
                socket.send(Message::Text(message)).await?;
            }
            current = new_status;
        } else {
            trace!("Ignoring status update seq {}.", new_status.sequence);
        }
    }
}

pub async fn stats(rooms: State<Rooms>) -> Json<Stats> {
    Json(rooms.stats().await)
}
