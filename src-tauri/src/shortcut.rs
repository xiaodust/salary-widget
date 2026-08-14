use windows::core::{HSTRING, Interface, PCWSTR};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::{CSIDL_DESKTOPDIRECTORY, IShellLinkW, SHGetFolderPathW, ShellLink};

/// 在桌面创建当前可执行文件的快捷方式
pub fn create_desktop_shortcut() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_path = exe.to_string_lossy().to_string();
    let work_dir = exe
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    unsafe {
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let need_uninit = hr.0 == 0;

        let result = (|| -> windows::core::Result<()> {
            let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
            link.SetPath(PCWSTR(HSTRING::from(exe_path.as_str()).as_ptr()))?;
            link.SetWorkingDirectory(PCWSTR(HSTRING::from(work_dir.as_str()).as_ptr()))?;
            link.SetDescription(PCWSTR(HSTRING::from("薪资实时显示").as_ptr()))?;

            let persist: IPersistFile = link.cast()?;

            let mut buf = [0u16; 260];
            SHGetFolderPathW(None, CSIDL_DESKTOPDIRECTORY as i32, None, 0, &mut buf)?;
            let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
            let desktop = String::from_utf16_lossy(&buf[..len]);
            let lnk_path = format!("{}\\SalaryWidget.lnk", desktop);
            persist.Save(PCWSTR(HSTRING::from(lnk_path.as_str()).as_ptr()), true)?;
            Ok(())
        })();

        if need_uninit {
            CoUninitialize();
        }
        result.map_err(|e| e.to_string())
    }
}
