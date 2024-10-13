use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use async_broadcast::RecvError;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use serde::Deserialize;
use tracing::{debug, info, instrument, warn};

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
) -> impl IntoResponse {
    // TODO: validate room name
    let room_name = form.0.room;
    upgrade.on_upgrade(move |socket| websocket(info.ip(), socket, rooms.0, room_name))
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
    let (mut receiver, initial_text) = room.lock().await.subscribe();

    // What the client currently sees
    let mut current = initial_text;
    if !current.is_empty() {
        socket.send(Message::Text(current.to_string())).await?;
    }

    loop {
        tokio::select! {
            // Receive update
            text = receiver.recv_direct() => {
                let text = match text {
                    Ok(text) => text,
                    Err(RecvError::Overflowed(_)) => continue,
                    Err(e) => return Err(e.into()),
                };
                if current != text {
                    current = text;
                    socket.send(Message::Text(current.to_string())).await?;
                }
            }
            // User typed something
            message = socket.recv() => {
                let Some(message) = message else {
                    debug!("Client closed connection.");
                    return Ok(()); // client closed connection
                };
                if let Message::Text(text) = message? {
                    let mut room = room.lock().await;
                    current = Arc::from(&text[..]);
                    room.write(text).await;
                }
            }
        }
    }
}
