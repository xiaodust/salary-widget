use windows_sys::Win32::Foundation::ERROR_FILE_NOT_FOUND;
use windows_sys::Win32::System::Registry::{
    HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ, RegCloseKey, RegDeleteValueW,
    RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
};

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const VALUE_NAME: &str = "SalaryWidget";

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn is_enabled() -> bool {
    unsafe {
        let key = wide(RUN_KEY);
        let mut hkey = std::ptr::null_mut();
        if RegOpenKeyExW(HKEY_CURRENT_USER, key.as_ptr(), 0, KEY_QUERY_VALUE, &mut hkey) != 0 {
            return false;
        }
        let name = wide(VALUE_NAME);
        let mut data = [0u16; 1024];
        let mut size = (data.len() * 2) as u32;
        let rc = RegQueryValueExW(
            hkey,
            name.as_ptr(),
            std::ptr::null(),
            std::ptr::null_mut(),
            data.as_mut_ptr() as *mut u8,
            &mut size,
        );
        RegCloseKey(hkey);
        rc == 0
    }
}

pub fn set_enabled(on: bool) -> Result<(), String> {
    unsafe {
        let key = wide(RUN_KEY);
        let mut hkey = std::ptr::null_mut();
        let rc = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            0,
            KEY_SET_VALUE,
            &mut hkey,
        );
        if rc != 0 {
            return Err(format!("打开注册表 Run 键失败（错误码 {rc}）"));
        }

        let name = wide(VALUE_NAME);
        let rc = if on {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let value = format!("\"{}\"", exe.display());
            let data = wide(&value);
            RegSetValueExW(
                hkey,
                name.as_ptr(),
                0,
                REG_SZ,
                data.as_ptr() as *const u8,
                (data.len() * 2) as u32,
            )
        } else {
            RegDeleteValueW(hkey, name.as_ptr())
        };
        RegCloseKey(hkey);

        if rc == 0 || (!on && rc == ERROR_FILE_NOT_FOUND) {
            Ok(())
        } else {
            Err(format!("写入注册表失败（错误码 {rc}）"))
        }
    }
}
