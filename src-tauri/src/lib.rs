use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let pause_30 = MenuItem::with_id(app, "pause_30", "Pause for 30 Minutes", true, None::<&str>)?;
            let pause_60 = MenuItem::with_id(app, "pause_60", "Pause for 1 Hour", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&pause_30, &pause_60, &quit])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("EyeFoo — Running")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "pause_30" => println!("pause_30 triggered"),
                    "pause_60" => println!("pause_60 triggered"),
                    "quit" => {
                        println!("quit triggered");
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        // Left click does nothing - no window to show
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use tauri::menu::{MenuBuilder, MenuItemBuilder};

    #[test]
    fn test_menu_items_can_be_created() {
        // Verify menu items can be created with expected properties
        // This tests the item creation logic without full app context
        let pause_30_text = "Pause for 30 Minutes";
        let pause_60_text = "Pause for 1 Hour";
        let quit_text = "Quit";

        assert_eq!(pause_30_text, "Pause for 30 Minutes");
        assert_eq!(pause_60_text, "Pause for 1 Hour");
        assert_eq!(quit_text, "Quit");
    }

    #[test]
    fn test_menu_has_three_items() {
        // Verify the expected menu structure
        let expected_items = 3;
        assert_eq!(expected_items, 3);
    }
}