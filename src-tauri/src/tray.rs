use crate::{autostart, commands, config, shortcut};
use std::sync::Mutex;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

fn current_config(app: &AppHandle) -> config::Config {
    let state = app.state::<Mutex<config::Config>>();
    state
        .lock()
        .map(|g| g.clone())
        .unwrap_or_default()
}

fn toggle_main_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
        } else {
            let _ = w.show();
            let _ = w.set_focus();
        }
    }
}

pub fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| tauri::Error::AssetNotFound("default icon".into()))?;

    TrayIconBuilder::with_id("main-tray")
        .tooltip("薪资实时显示")
        .icon(icon)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle" => toggle_main_window(app),
            "settings" => {
                // 锁定状态下打开设置先自动解锁，保证设置可用
                let cfg = current_config(app);
                if cfg.locked {
                    let _ = commands::set_locked(app.clone(), false);
                }
                let _ = app.emit("open-settings", ());
            }
            "lock" => {
                let cfg = current_config(app);
                let _ = commands::set_locked(app.clone(), !cfg.locked);
                let _ = rebuild_menu(app);
            }
            "shortcut" => {
                let _ = shortcut::create_desktop_shortcut();
            }
            "autostart" => {
                let mut cfg = current_config(app);
                cfg.autostart = !cfg.autostart;
                let _ = autostart::set_enabled(cfg.autostart);
                let _ = config::save(app, &cfg);
                let state = app.state::<Mutex<config::Config>>();
                *state.lock().unwrap() = cfg;
                let _ = rebuild_menu(app);
            }
            "mode_topmost" => {
                let _ = commands::set_display_mode(app.clone(), "topmost".into());
                let _ = rebuild_menu(app);
            }
            "mode_desktop" => {
                let _ = commands::set_display_mode(app.clone(), "desktop".into());
                let _ = rebuild_menu(app);
            }
            "mode_normal" => {
                let _ = commands::set_display_mode(app.clone(), "normal".into());
                let _ = rebuild_menu(app);
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    rebuild_menu(app)
}

fn rebuild_menu(app: &AppHandle) -> tauri::Result<()> {
    let cfg = current_config(app);

    let toggle = MenuItem::with_id(app, "toggle", "显示 / 隐藏", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    let shortcut_item =
        MenuItem::with_id(app, "shortcut", "创建桌面快捷方式", true, None::<&str>)?;
    let autostart_item = CheckMenuItem::with_id(
        app,
        "autostart",
        "开机自启",
        true,
        cfg.autostart,
        None::<&str>,
    )?;

    let m_top = CheckMenuItem::with_id(
        app,
        "mode_topmost",
        "置顶模式",
        true,
        cfg.display_mode == "topmost",
        None::<&str>,
    )?;
    let m_desk = CheckMenuItem::with_id(
        app,
        "mode_desktop",
        "桌面模式",
        true,
        cfg.display_mode == "desktop",
        None::<&str>,
    )?;
    let m_norm = CheckMenuItem::with_id(
        app,
        "mode_normal",
        "普通模式",
        true,
        cfg.display_mode == "normal",
        None::<&str>,
    )?;
    let mode_menu = Submenu::with_items(app, "显示模式", true, &[&m_top, &m_desk, &m_norm])?;
    let lock_item = CheckMenuItem::with_id(
        app,
        "lock",
        if cfg.locked {
            "解锁（恢复交互）"
        } else {
            "锁定（点击穿透）"
        },
        true,
        cfg.locked,
        None::<&str>,
    )?;

    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &toggle,
            &mode_menu,
            &lock_item,
            &settings,
            &shortcut_item,
            &autostart_item,
            &sep,
            &quit,
        ],
    )?;

    if let Some(tray) = app.tray_by_id("main-tray") {
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}
