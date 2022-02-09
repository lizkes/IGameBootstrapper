use native_windows_gui as nwg;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

use crate::lib::api::{get_download_url, ProviderGroup};
use crate::lib::file::{
    extract_tzst, get_random_temp_dir_path, get_temp_file_path, try_remove_path,
};
use crate::lib::net::{download_file, download_file_without_notice};
use crate::static_var;

pub fn depend_is_installed(name: &str) -> bool {
    match name {
        ".NET框架 4.8" => {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let net_framwork_key =
                match hklm.open_subkey(r"SOFTWARE\Microsoft\NET Framework Setup\NDP\v4\Full") {
                    Ok(v) => v,
                    Err(_) => {
                        return false;
                    }
                };
            let release_version: u32 = match net_framwork_key.get_value("Release") {
                Ok(v) => v,
                Err(_) => {
                    return false;
                }
            };

            if release_version >= 528040 {
                return true;
            }

            return false;
        }
        "WebView2" => {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let webview2_path: &str;
            if *static_var::OS_ARCH == 64 {
                webview2_path = r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
            } else {
                webview2_path =
                    r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
            }
            let webview2_key = match hklm.open_subkey(webview2_path) {
                Ok(v) => v,
                Err(_) => {
                    return false;
                }
            };
            let webview2_version: String = match webview2_key.get_value("pv") {
                Ok(v) => v,
                Err(_) => {
                    return false;
                }
            };

            if webview2_version != "" {
                return true;
            } else {
                return false;
            }
        }
        "IGame安装器" => {
            let igame_path = r"C:\Program Files\Infinite Dreams\IGameInstaller\IGameInstaller.exe";
            let path = std::path::Path::new(igame_path);
            return path.exists();
        }
        _ => {
            return false;
        }
    }
}

pub fn download_depend(
    name: &str,
    rate: Arc<Mutex<u8>>,
    rate_sender: nwg::NoticeSender,
) -> Result<(), String> {
    let url: String;
    let file_name: &str;
    match name {
        ".NET框架 4.8" => {
            // 下载证书修复工具
            let cert_fixer_url = get_download_url(13, &ProviderGroup::Fast);
            download_file_without_notice(cert_fixer_url.as_str(), "Rootsupd.tzst")?;

            url = get_download_url(9, &ProviderGroup::Fast);
            file_name = ".NET Framework 4.8.tzst";
        }
        "WebView2" => {
            if *static_var::OS_ARCH == 64 {
                url = get_download_url(11, &ProviderGroup::Fast);
            } else {
                url = get_download_url(10, &ProviderGroup::Fast);
            }
            file_name = "WebView2Installer.tzst";
        }
        "IGame安装器" => {
            url = get_download_url(12, &ProviderGroup::Fast);
            file_name = "IGameInstaller.tzst";
        }
        _ => {
            return Err("依赖名称不正确".to_string());
        }
    }

    return download_file(url.as_str(), file_name, rate, rate_sender);
}

pub fn install_depend(name: &str) -> Result<(), String> {
    match name {
        ".NET框架 4.8" => {
            //  安装Rootsupd
            let tzst_path = get_temp_file_path("Rootsupd.tzst");
            let dst_dir = get_random_temp_dir_path();

            extract_tzst(&tzst_path, &dst_dir)?;
            try_remove_path(&tzst_path)?;
            let mut installer_path = dst_dir.clone();
            installer_path.push("Rootsupd.exe");
            let output = std::process::Command::new(installer_path).output().unwrap();

            if output.status.success() {
                try_remove_path(&dst_dir)?;
            } else {
                return Err(std::str::from_utf8(&output.stderr).unwrap().to_string());
            }

            //  安装.NET Framework 4.8
            let tzst_path = get_temp_file_path(".NET Framework 4.8.tzst");
            let dst_dir = get_random_temp_dir_path();

            extract_tzst(&tzst_path, &dst_dir)?;
            try_remove_path(&tzst_path)?;
            let mut installer_path = dst_dir.clone();
            installer_path.push(".NET Framework 4.8.exe");
            let output = std::process::Command::new(installer_path)
                .args(["/passive", "/showrmui", "/promptrestart"])
                .output()
                .unwrap();

            if !output.status.success() {
                return Err(format!(
                    ".NET框架 4.8安装报告了一个错误: {}",
                    std::str::from_utf8(&output.stderr).unwrap().to_string()
                ));
            }
            try_remove_path(&dst_dir)?;
        }
        "WebView2" => {
            let tzst_path = get_temp_file_path("WebView2Installer.tzst");
            let dst_dir = get_random_temp_dir_path();

            extract_tzst(&tzst_path, &dst_dir)?;
            try_remove_path(&tzst_path)?;
            let mut installer_path = dst_dir.clone();
            if *static_var::OS_ARCH == 64 {
                installer_path.push("WebView2RuntimeInstallerX64.exe");
            } else {
                installer_path.push("WebView2RuntimeInstallerX32.exe");
            }
            let output = std::process::Command::new(installer_path)
                .args(["/silent", "/install"])
                .output()
                .unwrap();

            if !output.status.success() {
                return Err(format!(
                    "WebView2安装报告了一个错误: {}",
                    std::str::from_utf8(&output.stderr).unwrap().to_string()
                ));
            }
            try_remove_path(&dst_dir)?;
        }
        "IGame安装器" => {
            let tzst_path = get_temp_file_path("IGameInstaller.tzst");
            let dst_dir =
                PathBuf::from_str(r"C:\Program Files\Infinite Dreams\IGameInstaller").unwrap();

            extract_tzst(&tzst_path, &dst_dir)?;
            try_remove_path(&tzst_path)?;
        }
        _ => {
            return Err("依赖名称不正确".to_string());
        }
    }

    return Ok(());
}

// pub fn get_net_framwork_version() -> String {
//     let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
//     let net_framwork_key =
//         match hklm.open_subkey(r"SOFTWARE\Microsoft\NET Framework Setup\NDP\v4\Full") {
//             Ok(v) => v,
//             Err(_) => {
//                 return "unknown".to_string();
//             }
//         };
//     let release_version: u32 = match net_framwork_key.get_value("Release") {
//         Ok(v) => v,
//         Err(_) => {
//             return "unknown".to_string();
//         }
//     };

//     if release_version >= 528040 {
//         return "4.8".to_string();
//     } else if release_version >= 461808 {
//         return "4.7.2".to_string();
//     } else if release_version >= 461308 {
//         return "4.7.1".to_string();
//     } else if release_version >= 460798 {
//         return "4.7".to_string();
//     } else if release_version >= 394802 {
//         return "4.6.2".to_string();
//     } else if release_version >= 394254 {
//         return "4.6.1".to_string();
//     } else if release_version >= 393295 {
//         return "4.6".to_string();
//     } else if release_version >= 379893 {
//         return "4.5.2".to_string();
//     } else if release_version >= 378675 {
//         return "4.5.1".to_string();
//     } else if release_version >= 378389 {
//         return "4.5".to_string();
//     } else {
//         return "unknown".to_string();
//     }
// }
