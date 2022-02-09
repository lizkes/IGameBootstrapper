use lazy_static::lazy_static;
use std::time::Duration;
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

lazy_static! {
    pub static ref UREQ_AGENT: ureq::Agent = {
        return ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(30))
            .max_idle_connections_per_host(10)
            .build();
    };
    pub static ref CURRENT_EXE_PATH: String = {
        return std::env::current_exe()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap();
    };
    pub static ref CURRENT_DIR_PATH: String = {
        return std::env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap();
    };
    pub static ref OS_ARCH: u8 = {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let environment_key = match hklm
            .open_subkey(r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment")
        {
            Ok(v) => v,
            Err(_) => {
                return 32;
            }
        };
        let arch: String = match environment_key.get_value("PROCESSOR_ARCHITECTURE") {
            Ok(v) => v,
            Err(_) => {
                return 32;
            }
        };

        if arch == "x86" {
            return 32;
        } else {
            return 64;
        }
    };
}
