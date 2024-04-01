// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(string_remove_matches)]

mod device;
mod server;
mod system_tray;
mod accumulator;
mod kernel;

use device::Device;
use ngrok::tunnel::UrlTunnel;
use server::server;
use system_tray::{create_system_tray, on_system_tray_event};

use crate::server::start_tunnel;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn get_env(name: &str) -> String {
    std::env::var(String::from(name)).unwrap_or(String::from(""))
}

#[tokio::main]
fn main() {
    dotenvy::dotenv().expect(".env file not found");

    //! static_ngrok_domain
    let server_url = "https://balanced-tiger-hardy.ngrok-free.app".to_string();
    
    // TODO : uncomment and use later, after open-interpreter core rewrite in rust
    // let http_tunnel = futures::executor::block_on(start_tunnel()).unwrap();
    // let server_url = http_tunnel.url().to_owned();

    let mut app = tauri::Builder::default()
        .manage(Device::new(server_url))
        .invoke_handler(tauri::generate_handler![get_env])
        .system_tray(create_system_tray())
        .on_system_tray_event(on_system_tray_event)
        .setup(|_app| {
            tauri::async_runtime::spawn(server(http_tunnel));
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        },
        _ => {},
    });
}
