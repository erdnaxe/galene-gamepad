use anyhow::Result;
use clap::Parser;

mod galene_client;
mod virtual_controller;

#[derive(Parser, Debug)]
#[command(author, version)]
#[command(about = "Virtual gamepad controller for Galene", long_about = None)]
struct Cli {
    /// Galene WebSocket server address, example: "wss://galene.example.com/ws"
    #[arg(short, long)]
    server: String,

    /// Group name
    #[arg(short, long)]
    group: String,

    /// Group username
    #[arg(short, long, default_value = "Virtual GamePad")]
    username: String,

    /// Group password
    #[arg(short, long, default_value = "")]
    password: String,
}

fn main() -> Result<()> {
    // Log level INFO by default, can be overriden using RUST_LOG environment
    // variable
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    // Generate a new random client identifier
    let client_id = uuid::Uuid::new_v4();

    let mut device = virtual_controller::setup()?;
    let mut device_press_release = |type_: evdev::EventType, code: u16, value: i32, delay: u64| {
        virtual_controller::press_release(&mut device, type_, code, value, delay)
    };
    let mut socket = galene_client::connect(
        &cli.server,
        &client_id,
        &cli.group,
        &cli.username,
        &cli.password,
    )?;
    loop {
        galene_client::handle_message(&mut socket, &mut device_press_release)?
    }
}
