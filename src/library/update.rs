use native_windows_gui as nwg;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use version_compare::{compare_to, Cmp};

use crate::library::api::{get_download_url, get_resourc_version, ProviderGroup};
use crate::library::file::{
    extract_tzst, get_random_temp_dir_path, get_temp_file_path, try_copy_file, try_move_file,
    try_remove_path, write_resource_id_to_file,
};
use crate::library::net::download_file;
use crate::library::process::{exit, start_exe_as_admin};
use crate::static_var;

pub fn need_update() -> bool {
    let remote_version = get_resourc_version(8);
    return compare_to(remote_version, env!("CARGO_PKG_VERSION"), Cmp::Gt).unwrap();
}

pub fn download_update(rate: Arc<Mutex<u8>>, rate_sender: nwg::NoticeSender) -> Result<(), String> {
    let download_url = get_download_url(8, &ProviderGroup::Fast);
    let file_name = "IGameBootstrapper.tzst";
    download_file(download_url.as_str(), file_name, rate, rate_sender)?;
    return Ok(());
}

pub fn install_update(resource_id: i32) -> Result<(), String> {
    let self_exe_path_string = (*static_var::CURRENT_EXE_PATH).clone();
    let self_exe_path = PathBuf::from(self_exe_path_string.clone());
    let tzst_path = get_temp_file_path("IGameBootstrapper.tzst");
    let dst_dir = get_random_temp_dir_path();
    extract_tzst(&tzst_path, &dst_dir)?;
    try_remove_path(&tzst_path)?;
    let mut download_exe_path = dst_dir.clone();
    download_exe_path.push("IGameBootstrapper.exe");
    let old_version_file_path = PathBuf::from(format!(
        "{}_old.exe",
        self_exe_path_string[..self_exe_path_string.len() - 4].to_string()
    ));
    try_move_file(&self_exe_path, &old_version_file_path)?;
    try_copy_file(&download_exe_path, &self_exe_path)?;
    try_remove_path(&dst_dir)?;
    write_resource_id_to_file(&self_exe_path, resource_id)?;
    start_exe_as_admin(
        &self_exe_path_string,
        (*static_var::CURRENT_DIR_PATH).as_str(),
        "",
    );
    exit(0);

    return Ok(());
}

pub fn try_clean_old_version_file() -> Result<(), String> {
    let self_exe_path_string = (*static_var::CURRENT_EXE_PATH).clone();
    let old_version_file_path = PathBuf::from(format!(
        "{}_old.exe",
        self_exe_path_string[..self_exe_path_string.len() - 4].to_string()
    ));
    if old_version_file_path.exists() {
        try_remove_path(&old_version_file_path)?;
    }

    return Ok(());
}
