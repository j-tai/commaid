use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use async_broadcast::RecvError;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use tokio::sync::Mutex;
use tracing::{debug, info, instrument, warn};

use crate::room::{Room, Rooms};
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
    if !matches!(room_name.len(), 6..=16) || !room_name.chars().all(|c| c.is_ascii_alphanumeric()) {
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
    socket
        .send(Message::Text(current.encode(&Status::default())))
        .await?;

    loop {
        tokio::select! {
            // Received update
            result = receiver.recv_direct() => {
                let new_status = match result {
                    Ok(new_status) => new_status,
                    Err(RecvError::Overflowed(_)) => continue,
                    Err(e) => return Err(e.into()),
                };
                if new_status > current {
                    socket.send(Message::Text(new_status.encode(&current))).await?;
                    current = new_status;
                }
            }
            // User typed something
            message = socket.recv() => {
                let Some(message) = message else {
                    debug!("Client closed connection.");
                    return Ok(()); // client closed connection
                };
                let Message::Text(text) = message? else {
                    continue;
                };
                current = room.lock().await.write(text).await;
                continue;
            }
        }
    }
}
