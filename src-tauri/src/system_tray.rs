use tauri::{
    api::shell::open, AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent,
    SystemTrayMenu, SystemTrayMenuItem, SystemTraySubmenu,
};

const LINKS: [(&str, &str, &str); 2] = [
    // github links
    ("open-github-source", "OS1", "https://github.com/cs50victor/os1"),
    ("open-send-feedback", "Send Feedback", "https://dm.new/vic"),
];

pub fn create_system_tray() -> SystemTray {
    let sub_menu_github = {
        let mut menu = SystemTrayMenu::new();
        for (id, label, _url) in
            LINKS.iter().filter(|(id, _label, _url)| id.starts_with("open-github"))
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

    SystemTray::new().with_menu(tray_menu)
}

pub fn on_system_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    if let SystemTrayEvent::MenuItemClick { id, .. } = event {
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
                        let _ = window.show();
                        item_handle.set_title("Hide").unwrap();
                    },
                    Err(_e) => unimplemented!("what kind of errors happen here?"),
                }
            },
            "quit" => app.exit(0),
            s if s.starts_with("open-") => {
                if let Some(link) = LINKS.iter().find(|(id, ..)| id == &s) {
                    open(&app.shell_scope(), link.2, None).unwrap();
                }
            },
            _ => {},
        }
    }
}
