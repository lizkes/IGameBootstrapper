use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;

use crate::lib::error::process_error;

pub fn windows_ptr(value: &str) -> Vec<u16> {
    return OsStr::new(value).encode_wide().chain(once(0)).collect();
}

pub fn start_exe_as_admin(exe_path: &str, dir_path: &str, args: &str) {
    let ret = unsafe {
        winapi::um::shellapi::ShellExecuteW(
            std::ptr::null_mut(),
            windows_ptr("runas").as_ptr(),
            windows_ptr(exe_path).as_ptr(),
            windows_ptr(args).as_ptr(),
            windows_ptr(dir_path).as_ptr(),
            1,
        )
    };

    if (ret as usize) != 5 && (ret as usize) <= 32 {
        process_error(
            std::io::Error::last_os_error().to_string(),
            true,
            true,
            true,
            true,
        );
    }
}

pub fn start_igame_installer(resource_id: i32) {
    let exe_path = r"C:\Program Files\Infinite Dreams\IGameInstaller\IGameInstaller.exe";
    let dir_path = r"C:\Program Files\Infinite Dreams\IGameInstaller";
    let args = format!("\"{}\"", resource_id);

    start_exe_as_admin(exe_path, dir_path, args.as_str());
}

pub fn exit(code: i32) {
    std::process::exit(code);
}
