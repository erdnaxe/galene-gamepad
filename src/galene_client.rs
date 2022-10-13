use anyhow::{bail, Result};
use log::{debug, info, warn};
use serde_json::json;
use std::net::TcpStream;
use tungstenite::{stream::MaybeTlsStream, WebSocket};

/// Create new WebSocket connection to Galene server then join a room
pub fn connect(
    server: &str,
    client_id: &uuid::Uuid,
    group: &str,
    username: &str,
    password: &str,
) -> Result<WebSocket<MaybeTlsStream<TcpStream>>> {
    // Connect to WebSocket
    let server = url::Url::parse(server)?;
    let (mut socket, _) = tungstenite::connect(server)?;

    // Handshake with server
    debug!("Handshaking");
    let msg = json!({
        "type": "handshake",
        "version": ["1"],
        "id": client_id,
    });
    socket.write_message(msg.to_string().into())?;
    socket.read_message()?;

    // Join group
    info!("Joining group \"{group}\" as \"{username}\"");
    let msg = json!({
        "type": "join",
        "kind": "join",
        "group": group,
        "username": username,
        "password": password,
    });
    socket.write_message(msg.to_string().into())?;

    Ok(socket)
}

/// Handle incoming message from Galene WebSocket
pub fn handle_message<F>(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    mut device_cb: F,
) -> Result<()>
where
    F: FnMut(evdev::EventType, u16, i32, u64) -> std::io::Result<()>,
{
    use evdev::{AbsoluteAxisType, EventType, Key};

    let ws_msg = socket.read_message()?;
    let text_msg = ws_msg.to_text()?;
    let msg: serde_json::Value = serde_json::from_str(text_msg)?;
    let msg_type = msg["type"].as_str().unwrap_or_default();
    match msg_type {
        "ping" => {
            // Need to answer pong to ping request to keep connection
            let msg = json!({ "type": "pong" });
            socket
                .write_message(msg.to_string().into())
                .expect("Error sending pong message");
        }
        "usermessage" => {
            // Server is sending us a message
            let value = msg["value"].as_str().unwrap_or_default();
            match msg["kind"].as_str().unwrap_or_default() {
                "error" => bail!("Server returned error: {value}"),
                _ => warn!("Ignoring unimplemented {msg_type} message: {msg:?}"),
            }
        }
        "joined" => {
            // Response to the group join request
            match msg["kind"].as_str().unwrap_or_default() {
                "join" => debug!("Joined group"),
                "change" => debug!("Group configuration changed"),
                _ => bail!("Error joining group, please check group name, username and password."),
            }
        }
        "chat" => {
            // New chat message
            let value = msg["value"].as_str().unwrap_or_default().to_lowercase();
            let value = value.as_str();
            info!("Received: {value}");
            // TODO: if message has multiple letters, press multiple buttons at
            // the same time
            // TODO: parse this mapping from a TOML file
            match value {
                "z" => device_cb(EventType::ABSOLUTE, AbsoluteAxisType::ABS_HAT0Y.0, -1, 300)?,
                "q" => device_cb(EventType::ABSOLUTE, AbsoluteAxisType::ABS_HAT0X.0, -1, 300)?,
                "s" => device_cb(EventType::ABSOLUTE, AbsoluteAxisType::ABS_HAT0Y.0, 1, 300)?,
                "d" => device_cb(EventType::ABSOLUTE, AbsoluteAxisType::ABS_HAT0X.0, 1, 300)?,
                "a" => device_cb(EventType::KEY, Key::BTN_SOUTH.code(), 1, 300)?,
                "b" => device_cb(EventType::KEY, Key::BTN_EAST.code(), 1, 300)?,
                "x" => device_cb(EventType::KEY, Key::BTN_NORTH.code(), 1, 300)?,
                "y" => device_cb(EventType::KEY, Key::BTN_WEST.code(), 1, 300)?,
                "start" => device_cb(EventType::KEY, Key::BTN_START.code(), 1, 300)?,
                "select" => device_cb(EventType::KEY, Key::BTN_SELECT.code(), 1, 300)?,
                "tl" => device_cb(EventType::KEY, Key::BTN_TL.code(), 1, 300)?,
                "tr" => device_cb(EventType::KEY, Key::BTN_TR.code(), 1, 300)?,
                _ => {}
            }
        }
        "abort" | "answer" | "ice" | "renegotiate" | "user" | "chathistory" | "close" => {
            // Ignore as we do not stream media
            debug!("Ignoring {msg_type} message");
        }
        _ => {
            warn!("Ignoring unimplemented {msg_type} message: {msg:?}");
        }
    }
    Ok(())
}
