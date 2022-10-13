use anyhow::Result;

mod galene_client;
mod virtual_controller;

fn main() -> Result<()> {
    // Log level INFO by default, can be overriden using RUST_LOG environment
    // variable
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // TODO: configuration file
    let server = "wss://galene.crans.org/ws";
    let group = "CHANGE_ME";
    let username = "Virtual GamePad";
    let password = "";

    // Generate a new random client identifier
    let client_id = uuid::Uuid::new_v4();

    let mut device = virtual_controller::setup()?;
    let mut device_press_release = |type_: evdev::EventType, code: u16, value: i32, delay: u64| {
        virtual_controller::press_release(&mut device, type_, code, value, delay)
    };
    let mut socket = galene_client::connect(server, &client_id, group, username, password)?;
    loop {
        galene_client::handle_message(&mut socket, &mut device_press_release)?
    }
}
