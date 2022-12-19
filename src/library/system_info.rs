use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

pub fn os_is_ok() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let current_version_key =
        match hklm.open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion") {
            Ok(v) => v,
            Err(_) => {
                return false;
            }
        };
    let current_version_value: String = match current_version_key.get_value("CurrentVersion") {
        Ok(v) => v,
        Err(_) => {
            return false;
        }
    };

    // 必须是win7 win8.1 win10 win11
    if current_version_value.starts_with("6.")
        && (current_version_value != "6.0" || current_version_value != "6.2")
    {
        return true;
    }

    return false;
}
