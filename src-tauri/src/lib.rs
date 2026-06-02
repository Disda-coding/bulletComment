mod commands;
mod server;
mod tray;

use commands::ServerState;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ServerState {
            running: AtomicBool::new(false),
            port: Mutex::new(9090),
        })
        .setup(|app| {
            tray::setup_tray(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_server,
            commands::stop_server,
            commands::get_local_ip,
            commands::get_server_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
