mod autostart;
mod calc;
mod commands;
mod config;
mod desktop;
mod shortcut;
mod single_instance;
mod tray;

use std::sync::Mutex;
use tauri::{Emitter, Manager, WindowEvent};

pub fn run() {
    if !single_instance::acquire() {
        return;
    }

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            let cfg = config::load(&app_handle).unwrap_or_default();
            app.manage(Mutex::new(cfg.clone()));
            if cfg.autostart != autostart::is_enabled() {
                let _ = autostart::set_enabled(cfg.autostart);
            }

            let main_win = app
                .get_webview_window("main")
                .expect("main window should exist");

            let mut effective_cfg = cfg.clone();
            if let Err(_) = commands::apply_display_mode(&app_handle, &cfg.display_mode) {
                effective_cfg.display_mode = "normal".into();
                if config::save(&app_handle, &effective_cfg).is_ok() {
                    if let Ok(mut guard) = app_handle.state::<Mutex<config::Config>>().lock() {
                        *guard = effective_cfg.clone();
                    }
                }
                let _ = main_win.set_always_on_top(false);
                let _ = app_handle.emit("desktop-unavailable", ());
            }
            commands::restore_position(&app_handle, &main_win)
                .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
            let _ = main_win.show();
            let _ = main_win.set_focus();
            // 若上次退出时处于锁定状态，启动后继续穿透
            if cfg.locked {
                let _ = main_win.set_ignore_cursor_events(true);
            }

            tray::create_tray(&app_handle)
                .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

            // 每秒推送一次薪资快照
            let h = app_handle.clone();
            std::thread::spawn(move || loop {
                let snap = {
                    let state = h.state::<Mutex<config::Config>>();
                    let cfg = state.lock().map(|g| g.clone()).unwrap_or_default();
                    calc::snapshot(&cfg)
                };
                let _ = h.emit("salary:tick", snap);
                std::thread::sleep(std::time::Duration::from_secs(1));
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { .. } = event {
                if window.label() == "main" {
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::set_display_mode,
            commands::save_position,
            commands::create_shortcut,
            commands::set_autostart,
            commands::set_locked,
            commands::quit_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
