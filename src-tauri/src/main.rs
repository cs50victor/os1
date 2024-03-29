// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod server;
mod device;

use server::server;
use tauri::{
    api::shell::open, AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent,
    SystemTrayMenu, SystemTrayMenuItem, SystemTraySubmenu,
};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn get_env(name: &str) -> String {
    std::env::var(String::from(name)).unwrap_or(String::from(""))
}

const links: [(&str, &str, &str); 2] = [
    // github links
    ("open-github-source", "OS1", "https://github.com/cs50victor/os1"),
    ("open-send-feedback", "Send Feedback", "https://dm.new/vic"),
];

fn main() {
    dotenvy::dotenv().expect(".env file not found");

    let sub_menu_github = {
        let mut menu = SystemTrayMenu::new();
        for (id, label, _url) in
            links.iter().filter(|(id, label, _url)| id.starts_with("open-github"))
        {
            menu = menu.add_item(CustomMenuItem::new(id.to_string(), label.to_string()));
        }

        SystemTraySubmenu::new("GitHub", menu)
    };

    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("quit".to_string(), "Quit"))
        .add_submenu(sub_menu_github)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("open-send-feedback".to_string(), "Send Feedback"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("visibility-toggle".to_string(), "Hide"));

    let tray = SystemTray::new().with_menu(tray_menu);

    let mut app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_env])
        .system_tray(tray)
        .on_system_tray_event(on_system_tray_event)
        .setup(|app|{
            tauri::async_runtime::spawn(server());
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

fn on_system_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => {
            let item_handle = app.tray_handle().get_item(&id);
            dbg!(&id);
            match id.as_str() {
                "visibility-toggle" => {
                    let window = app.get_window("main").unwrap();
                    match window.is_visible() {
                        Ok(true) => {
                            window.hide().unwrap();
                            item_handle.set_title("Show").unwrap();
                        },
                        Ok(false) => {
                            window.show();
                            item_handle.set_title("Hide").unwrap();
                        },
                        Err(e) => unimplemented!("what kind of errors happen here?"),
                    }
                },
                "quit" => app.exit(0),
                s if s.starts_with("open-") => {
                    if let Some(link) = links.iter().find(|(id, ..)| id == &s) {
                        open(&app.shell_scope(), link.2, None).unwrap();
                    }
                },
                _ => {},
            }
        },
        _ => {},
    }
}
