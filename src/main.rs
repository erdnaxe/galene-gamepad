mod galene_client;
mod virtual_controller;

fn main() {
    // Log level INFO by default, can be overriden using RUST_LOG environment
    // variable
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // TODO: configuration file
    let server = "wss://galene.crans.org/ws";
    let group = "change_me";
    let username = "Virtual GamePad";
    let password = "";

    // Generate a new random client identifier
    let client_id = uuid::Uuid::new_v4();

    let mut device = virtual_controller::setup().unwrap();
    let mut socket = galene_client::connect(server, &client_id, group, username, password).unwrap();
    loop {
        galene_client::handle_message(&mut socket, &mut device)
    }
}
