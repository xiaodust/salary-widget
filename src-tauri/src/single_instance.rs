use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowW, SetForegroundWindow, ShowWindow, SW_SHOW,
};

/// 单实例：重复启动时尝试唤起已有窗口并退出本实例。
pub fn acquire() -> bool {
    unsafe {
        let name: Vec<u16> = "Local\\SalaryWidget.SingleInstance"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let h = CreateMutexW(std::ptr::null(), 0, name.as_ptr());
        if h.is_null() {
            return true;
        }
        if GetLastError() == ERROR_ALREADY_EXISTS {
            let title: Vec<u16> = "薪资实时显示"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let w = FindWindowW(std::ptr::null(), title.as_ptr());
            if !w.is_null() {
                ShowWindow(w, SW_SHOW);
                SetForegroundWindow(w);
            }
            false
        } else {
            // 持有互斥句柄直到进程退出（不关闭即可）
            let _ = h;
            true
        }
    }
}
