#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Prevents additional console window on Windows in release, DO NOT REMOVE!!

use server::server_test::{server_start};
mod server;
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    #[cfg(debug_assertions)]
    let builder = tauri::Builder::default().plugin(devtools::init());
    
    #[cfg(not(debug_assertions))]
    let builder = tauri::Builder::default();
    
    server_start();

    builder
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    



}
