use tauri::WebviewWindow;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowExW, FindWindowW, SendMessageTimeoutW, SetParent, ShowWindow, SMTO_NORMAL, SW_SHOW,
};

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// 找到桌面壁纸层窗口（WorkerW）。
/// 返回的窗口位于壁纸之上、桌面图标之下、所有应用之下。
fn find_wallpaper_workerw() -> Option<HWND> {
    unsafe {
        let progman = FindWindowW(wide("Progman").as_ptr(), std::ptr::null());
        if progman.is_null() {
            return None;
        }
        let mut result: usize = 0;
        SendMessageTimeoutW(
            progman,
            0x052C,
            0,
            0,
            SMTO_NORMAL,
            1000,
            &mut result,
        );

        let mut worker = FindWindowExW(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            wide("WorkerW").as_ptr(),
            std::ptr::null(),
        );
        while !worker.is_null() {
            let defview = FindWindowExW(
                worker,
                std::ptr::null_mut(),
                wide("SHELLDLL_DefView").as_ptr(),
                std::ptr::null(),
            );
            if !defview.is_null() {
                let next = FindWindowExW(
                    std::ptr::null_mut(),
                    worker,
                    wide("WorkerW").as_ptr(),
                    std::ptr::null(),
                );
                // 下一个 WorkerW 即壁纸层；找不到则退回 Progman（图标层之上）
                return Some(if !next.is_null() { next } else { progman });
            }
            worker = FindWindowExW(
                std::ptr::null_mut(),
                worker,
                wide("WorkerW").as_ptr(),
                std::ptr::null(),
            );
        }
        Some(progman)
    }
}

/// 把窗口挂载到桌面层
pub fn attach(window: &WebviewWindow) -> bool {
    let hwnd = match window.hwnd() {
        Ok(h) => h.0,
        Err(_) => return false,
    };
    unsafe {
        if let Some(parent) = find_wallpaper_workerw() {
            SetParent(hwnd, parent);
            ShowWindow(hwnd, SW_SHOW);
            true
        } else {
            false
        }
    }
}

/// 从桌面层摘下，恢复为普通顶层窗口
pub fn detach(window: &WebviewWindow) {
    let hwnd = match window.hwnd() {
        Ok(h) => h.0,
        Err(_) => return,
    };
    unsafe {
        SetParent(hwnd, std::ptr::null_mut());
        ShowWindow(hwnd, SW_SHOW);
    }
}
