use anyhow::Result;
use evdev::uinput::VirtualDevice;
use log::{debug, info, warn};
use serde_json::json;
use std::net::TcpStream;
use tungstenite::stream::MaybeTlsStream;

use crate::virtual_controller::press_release;

/// Create new WebSocket connection to Galene server then join a room
pub fn connect(
    server: &str,
    client_id: &uuid::Uuid,
    group: &str,
    username: &str,
    password: &str,
) -> Result<tungstenite::WebSocket<MaybeTlsStream<TcpStream>>> {
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
pub fn handle_message(
    socket: &mut tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>,
    device: &mut VirtualDevice,
) -> Result<()> {
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
                "error" => panic!("Server returned error: {value}"),
                _ => warn!("Ignoring unimplemented {msg_type} message: {msg:?}"),
            }
        }
        "joined" => {
            // Response to the group join request
            match msg["kind"].as_str().unwrap_or_default() {
                "join" => debug!("Joined group"),
                "change" => debug!("Group configuration changed"),
                _ => panic!("Error joining group"),
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
                "z" => press_release(
                    device,
                    evdev::EventType::ABSOLUTE,
                    evdev::AbsoluteAxisType::ABS_HAT0Y.0,
                    -1,
                    300,
                )?,
                "q" => press_release(
                    device,
                    evdev::EventType::ABSOLUTE,
                    evdev::AbsoluteAxisType::ABS_HAT0X.0,
                    -1,
                    300,
                )?,
                "s" => press_release(
                    device,
                    evdev::EventType::ABSOLUTE,
                    evdev::AbsoluteAxisType::ABS_HAT0Y.0,
                    1,
                    300,
                )?,
                "d" => press_release(
                    device,
                    evdev::EventType::ABSOLUTE,
                    evdev::AbsoluteAxisType::ABS_HAT0X.0,
                    1,
                    300,
                )?,
                "a" => press_release(
                    device,
                    evdev::EventType::KEY,
                    evdev::Key::BTN_SOUTH.code(),
                    1,
                    300,
                )?,
                "b" => press_release(
                    device,
                    evdev::EventType::KEY,
                    evdev::Key::BTN_EAST.code(),
                    1,
                    300,
                )?,
                "x" => press_release(
                    device,
                    evdev::EventType::KEY,
                    evdev::Key::BTN_NORTH.code(),
                    1,
                    300,
                )?,
                "y" => press_release(
                    device,
                    evdev::EventType::KEY,
                    evdev::Key::BTN_WEST.code(),
                    1,
                    300,
                )?,
                "start" => press_release(
                    device,
                    evdev::EventType::KEY,
                    evdev::Key::BTN_START.code(),
                    1,
                    300,
                )?,
                "select" => press_release(
                    device,
                    evdev::EventType::KEY,
                    evdev::Key::BTN_SELECT.code(),
                    1,
                    300,
                )?,
                "tl" => press_release(
                    device,
                    evdev::EventType::KEY,
                    evdev::Key::BTN_TL.code(),
                    1,
                    300,
                )?,
                "tr" => press_release(
                    device,
                    evdev::EventType::KEY,
                    evdev::Key::BTN_TR.code(),
                    1,
                    300,
                )?,
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
