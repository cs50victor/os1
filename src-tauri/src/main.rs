// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  dotenvy::dotenv().expect(".env file not found");

  pretty_env_logger::env_logger::init();
  
  tauri::Builder::default()
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
