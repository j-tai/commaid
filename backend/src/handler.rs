use std::error::Error;
use std::net::{IpAddr, SocketAddr};

use async_broadcast::RecvError;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use tracing::{debug, info, instrument, trace, warn};

use crate::room::Rooms;

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

    // What the client currently sees
    let mut current = initial_status;
    if !current.text.is_empty() {
        socket.send(Message::Text(current.text.to_string())).await?;
    }

    loop {
        tokio::select! {
            // Receive update
            new_status = receiver.recv_direct() => {
                let new_status = match new_status {
                    Ok(s) => s,
                    Err(RecvError::Overflowed(_)) => continue,
                    Err(e) => return Err(e.into()),
                };
                if new_status > current {
                    trace!("Updating status, seq {} -> {}.", current.sequence, new_status.sequence);
                    current = new_status;
                    socket.send(Message::Text(current.text.to_string())).await?;
                }
            }
            // User typed something
            message = socket.recv() => {
                let Some(message) = message else {
                    debug!("Client closed connection.");
                    return Ok(()); // client closed connection
                };
                if let Message::Text(text) = message? {
                    let new_status = room.lock().await.write(text).await;
                    assert!(new_status > current);
                    current = new_status;
                }
            }
        }
    }
}
