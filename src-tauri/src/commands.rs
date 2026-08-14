use crate::{autostart, config, desktop, shortcut};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

fn current_config(app: &AppHandle) -> Result<config::Config, String> {
    let state = app.state::<Mutex<config::Config>>();
    state
        .lock()
        .map(|guard| guard.clone())
        .map_err(|e| e.to_string())
}

fn store_config(app: &AppHandle, cfg: config::Config) -> Result<(), String> {
    let state = app.state::<Mutex<config::Config>>();
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    *guard = cfg;
    Ok(())
}

fn emit_config(app: &AppHandle, cfg: config::Config) {
    let _ = app.emit("config-updated", cfg);
}

#[tauri::command]
pub fn get_config(app: AppHandle) -> Result<config::Config, String> {
    current_config(&app)
}

#[tauri::command]
pub fn save_config(app: AppHandle, config: config::Config) -> Result<(), String> {
    config.validate()?;
    let old = current_config(&app)?;
    let mode_changed = config.display_mode != old.display_mode;

    if mode_changed {
        apply_display_mode(&app, &config.display_mode)?;
    }

    if let Err(err) = config::save(&app, &config) {
        if mode_changed {
            let _ = apply_display_mode(&app, &old.display_mode);
        }
        return Err(err);
    }

    store_config(&app, config.clone())?;
    emit_config(&app, config);
    Ok(())
}

#[tauri::command]
pub fn set_display_mode(app: AppHandle, mode: String) -> Result<(), String> {
    if !matches!(mode.as_str(), "topmost" | "desktop" | "normal") {
        return Err("不支持的显示模式".into());
    }

    let old = current_config(&app)?;
    if old.display_mode == mode {
        return Ok(());
    }

    apply_display_mode(&app, &mode)?;

    let mut cfg = old.clone();
    cfg.display_mode = mode.clone();
    if let Err(err) = config::save(&app, &cfg) {
        let _ = apply_display_mode(&app, &old.display_mode);
        return Err(err);
    }

    store_config(&app, cfg.clone())?;
    emit_config(&app, cfg);
    Ok(())
}

#[tauri::command]
pub fn save_position(app: AppHandle, x: i32, y: i32) -> Result<(), String> {
    let mut cfg = current_config(&app)?;
    let (cx, cy) = if let Some(window) = app.get_webview_window("main") {
        let size = window.outer_size().map_err(|e| e.to_string())?;
        clamp_position(x, y, size.width as i32, size.height as i32)
    } else {
        (x, y)
    };
    cfg.pos_x = Some(cx);
    cfg.pos_y = Some(cy);

    config::save(&app, &cfg)?;
    store_config(&app, cfg.clone())?;
    emit_config(&app, cfg);
    Ok(())
}

#[tauri::command]
pub fn create_shortcut() -> Result<(), String> {
    shortcut::create_desktop_shortcut()
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let old = current_config(&app)?;
    if old.autostart == enabled {
        return Ok(());
    }

    autostart::set_enabled(enabled)?;

    let mut cfg = old.clone();
    cfg.autostart = enabled;
    if let Err(err) = config::save(&app, &cfg) {
        let _ = autostart::set_enabled(old.autostart);
        return Err(err);
    }

    store_config(&app, cfg.clone())?;
    emit_config(&app, cfg);
    Ok(())
}

#[tauri::command]
pub fn set_locked(app: AppHandle, enabled: bool) -> Result<(), String> {
    let old = current_config(&app)?;
    if old.locked == enabled {
        return Ok(());
    }

    if let Some(window) = app.get_webview_window("main") {
        window
            .set_ignore_cursor_events(enabled)
            .map_err(|e| e.to_string())?;
    }

    let mut cfg = old.clone();
    cfg.locked = enabled;
    if let Err(err) = config::save(&app, &cfg) {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.set_ignore_cursor_events(old.locked);
        }
        return Err(err);
    }

    store_config(&app, cfg.clone())?;
    emit_config(&app, cfg);
    Ok(())
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}

pub fn apply_display_mode(app: &AppHandle, mode: &str) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or("main window not found")?;
    match mode {
        "desktop" => {
            let pos = window.outer_position().ok();
            let ok = desktop::attach(&window);
            window.set_always_on_top(false).map_err(|e| e.to_string())?;
            if let Some(p) = pos {
                let _ = window.set_position(p);
            }
            if !ok {
                let _ = app.emit("desktop-unavailable", ());
                return Err("当前系统不支持桌面模式".into());
            }
        }
        "topmost" => {
            let pos = window.outer_position().ok();
            desktop::detach(&window);
            window.set_always_on_top(true).map_err(|e| e.to_string())?;
            if let Some(p) = pos {
                let _ = window.set_position(p);
            }
        }
        _ => {
            let pos = window.outer_position().ok();
            desktop::detach(&window);
            window.set_always_on_top(false).map_err(|e| e.to_string())?;
            if let Some(p) = pos {
                let _ = window.set_position(p);
            }
        }
    }
    Ok(())
}

pub fn restore_position(app: &AppHandle, window: &tauri::WebviewWindow) -> Result<(), String> {
    let cfg = current_config(app)?;
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let w = size.width as i32;
    let h = size.height as i32;
    let (cx, cy) = if let (Some(x), Some(y)) = (cfg.pos_x, cfg.pos_y) {
        clamp_position(x, y, w, h)
    } else {
        centered_position(w, h)
    };
    window
        .set_position(tauri::PhysicalPosition::new(cx, cy))
        .map_err(|e| e.to_string())
}

#[cfg(target_os = "windows")]
fn clamp_position(x: i32, y: i32, w: i32, h: i32) -> (i32, i32) {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };

    unsafe {
        let center = POINT {
            x: x + w / 2,
            y: y + h / 2,
        };
        let monitor = MonitorFromPoint(center, MONITOR_DEFAULTTONEAREST);
        if !monitor.is_null() {
            let mut info: MONITORINFO = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            if GetMonitorInfoW(monitor, &mut info) != 0 {
                let rect = (
                    info.rcWork.left,
                    info.rcWork.top,
                    info.rcWork.right,
                    info.rcWork.bottom,
                );
                return clamp_to_rect(rect, x, y, w, h);
            }
        }
    }

    clamp_to_rect(work_area(), x, y, w, h)
}

#[cfg(not(target_os = "windows"))]
fn clamp_position(x: i32, y: i32, w: i32, h: i32) -> (i32, i32) {
    clamp_to_rect(work_area(), x, y, w, h)
}

fn clamp_to_rect(rect: (i32, i32, i32, i32), x: i32, y: i32, w: i32, h: i32) -> (i32, i32) {
    let (left, top, right, bottom) = rect;
    let max_x = (right - w).max(left);
    let max_y = (bottom - h).max(top);
    (x.clamp(left, max_x), y.clamp(top, max_y))
}

fn work_area() -> (i32, i32, i32, i32) {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_GETWORKAREA};

    let mut wa = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    unsafe {
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            &mut wa as *mut RECT as *mut core::ffi::c_void,
            0,
        );
    }
    (wa.left, wa.top, wa.right, wa.bottom)
}

/// 没有可用显示器信息时，退回到主屏工作区居中。
fn centered_position(w: i32, h: i32) -> (i32, i32) {
    let (left, top, right, bottom) = work_area();
    let wa_w = (right - left).max(1);
    let wa_h = (bottom - top).max(1);
    (left + (wa_w - w) / 2, top + (wa_h - h) / 2)
}
