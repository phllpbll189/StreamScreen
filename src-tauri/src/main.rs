#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Prevents additional console window on Windows in release, DO NOT REMOVE!!

use server::server::server_start;
mod server;
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command


#[cfg(test)]
mod server_test;

fn main() {
    #[cfg(debug_assertions)]
    let builder = tauri::Builder::default().plugin(devtools::init());

    #[cfg(not(debug_assertions))]
    let builder = tauri::Builder::default();

    builder
        .setup(|app| {
            let app_handle = app.handle();
            std::thread::spawn(move || {
                server_start(app_handle);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
