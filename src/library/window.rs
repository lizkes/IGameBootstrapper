use crate::library::error::process_error;
use crate::library::process::windows_ptr;

pub fn open_error_message_box(message: &str) {
    use winapi::um::winuser::{MessageBoxW, MB_ICONERROR, MB_OK, MB_SETFOREGROUND};

    let ret = unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            windows_ptr(message).as_ptr(),
            windows_ptr("IGame引导程序错误").as_ptr(),
            MB_OK | MB_ICONERROR | MB_SETFOREGROUND,
        )
    };

    if ret == 0 {
        process_error(
            std::io::Error::last_os_error().to_string(),
            false,
            true,
            true,
            true,
        );
    }
}
