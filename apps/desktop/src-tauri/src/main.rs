// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Start the myday-server as a sidecar process
            let server_path = app
                .path()
                .resource_dir()
                .expect("failed to get resource dir")
                .join("myday-server");

            // Spawn the server process
            let _server_process = Command::new(server_path)
                .spawn()
                .expect("failed to start myday-server sidecar");

            // The server will be available on localhost:3001
            tracing::info!("Started myday-server sidecar");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
