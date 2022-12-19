#![allow(non_snake_case)]
#![windows_subsystem = "windows"]

mod library;
mod static_var;
mod ui;

use native_windows_gui as nwg;
use nwg::NativeUi;

use crate::library::depend;
use crate::library::error::process_error;
use crate::library::file::try_search_resource_id;
use crate::library::process::{exit, start_igame_installer};
use crate::library::system_info::os_is_ok;
use crate::library::ui::try_build_font;
use crate::library::update::{need_update, try_clean_old_version_file};
use crate::ui::{MainDlg, PromptDlg, UpdateDlg};

fn main() {
    // 检查系统是否满足要求
    if !os_is_ok() {
        process_error(
            "本软件只能运行在Win7 Win8.1 Win10 Win11系统上\n请尝试升级你的Windows系统".to_string(),
            true,
            true,
            false,
            true,
        );
    }

    // 字节检索自身来获取末尾的资源ID
    let resource_id = try_search_resource_id();

    // 删除过时的文件
    match try_clean_old_version_file() {
        Err(e) => process_error(format!("删除旧版本文件失败：{}", e), true, true, true, true),
        _ => {}
    };

    // 检查更新
    if need_update() {
        match nwg::init() {
            Err(e) => process_error(format!("初始化nwg失败：{}", e), true, true, true, true),
            _ => {}
        };
        let mut default_font = nwg::Font::default();
        try_build_font(15, "NSimSun", &mut default_font);
        nwg::Font::set_global_default(Some(default_font));
        let mut update_dlg: UpdateDlg = Default::default();
        update_dlg.set_resource_id(resource_id);
        let _update_dlg_ui = match UpdateDlg::build_ui(update_dlg) {
            Ok(v) => v,
            Err(e) => {
                process_error(format!("初始化UI失败：{}", e), true, true, true, true);
                return;
            }
        };
        nwg::dispatch_thread_events();
    }

    // 检查依赖
    let mut depends: Vec<&str> = Vec::new();
    if !depend::depend_is_installed(".NET框架 4.8") {
        depends.push(".NET框架 4.8");
    }
    if !depend::depend_is_installed("WebView2") {
        depends.push("WebView2");
    }
    if !depend::depend_is_installed("IGame安装器") {
        depends.push("IGame安装器");
    }

    // 需要安装依赖
    if depends.len() != 0 {
        match nwg::init() {
            Ok(_) => {}
            Err(e) => process_error(format!("初始化nwg失败：{}", e), true, true, true, true),
        };
        let mut default_font = nwg::Font::default();
        try_build_font(18, "NSimSun", &mut default_font);
        nwg::Font::set_global_default(Some(default_font));

        // 需要安装IGame安装器以外的依赖
        if depends.len() != 1 || depends[0] != "IGame安装器" {
            let mut prompt_dlg: PromptDlg = Default::default();
            prompt_dlg.set_needed_depends(&depends);
            let prompt_dlg_ui = match PromptDlg::build_ui(prompt_dlg) {
                Ok(v) => v,
                Err(e) => {
                    process_error(format!("初始化UI失败：{}", e), true, true, true, true);
                    return;
                }
            };
            nwg::dispatch_thread_events();
            drop(prompt_dlg_ui);
        }

        let mut main_dlg: MainDlg = Default::default();
        main_dlg.set_needed_depends(&depends);
        let _main_dlg_ui = match MainDlg::build_ui(main_dlg) {
            Ok(v) => v,
            Err(e) => {
                process_error(format!("初始化UI失败：{}", e), true, true, true, true);
                return;
            }
        };
        nwg::dispatch_thread_events();
    }

    // 启动IGame安装器
    start_igame_installer(resource_id);
    exit(0);
}
