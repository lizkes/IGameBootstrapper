use native_windows_gui as nwg;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::lib::depend::{download_depend, install_depend};
use crate::lib::error::process_error;
use crate::lib::process::exit;
use crate::lib::ui::try_build_font;

#[derive(Default)]
pub struct MainDlg {
    window: nwg::Window,
    window_icon: nwg::Icon,
    embed_resource: nwg::EmbedResource,

    prompt_label1: nwg::Label,
    prompt_label2: nwg::Label,
    download_description_label: nwg::Label,
    download_progressbar: nwg::ProgressBar,
    install_description_label: nwg::Label,
    install_progressbar: nwg::ProgressBar,

    download_description: Arc<Mutex<String>>,
    download_description_notice: nwg::Notice,
    download_rate: Arc<Mutex<u8>>,
    download_rate_notice: nwg::Notice,
    install_description: Arc<Mutex<String>>,
    install_description_notice: nwg::Notice,
    start_install_depend: Arc<Mutex<String>>,
    start_install_notice: nwg::Notice,
    install_done_depend: Arc<Mutex<String>>,
    install_done_notice: nwg::Notice,
    needed_depends: Vec<String>,
    installing_depends: Arc<Mutex<Vec<String>>>,
    installed_depends: Arc<Mutex<Vec<String>>>,
    is_error: Arc<Mutex<bool>>,
}

impl MainDlg {
    pub fn set_needed_depends(&mut self, depends: &Vec<&str>) {
        self.needed_depends = depends.iter().map(|s| s.to_string()).collect();
    }

    fn close(&self) {
        nwg::stop_thread_dispatch();
    }
}

pub struct MainDlgUi {
    inner: Rc<MainDlg>,
    default_handler: RefCell<Option<nwg::EventHandler>>,
}

impl nwg::NativeUi<MainDlgUi> for MainDlg {
    fn build_ui(mut dialog: Self) -> Result<MainDlgUi, nwg::NwgError> {
        use nwg::Event as E;

        let window_width = 600;
        let window_height = 224;

        // Resources
        nwg::EmbedResource::builder().build(&mut dialog.embed_resource)?;

        nwg::Icon::builder()
            .source_embed(Some(&dialog.embed_resource))
            .source_embed_str(Some("LOGO"))
            .build(&mut dialog.window_icon)?;

        // Controls
        nwg::Window::builder()
            .flags(
                nwg::WindowFlags::WINDOW
                    | nwg::WindowFlags::MINIMIZE_BOX
                    | nwg::WindowFlags::VISIBLE,
            )
            .size((window_width, window_height))
            .title(&format!("IGame引导器 v{}", env!("CARGO_PKG_VERSION")))
            .center(true)
            .icon(Some(&dialog.window_icon))
            .build(&mut dialog.window)?;

        let mut prompt_font = nwg::Font::default();
        try_build_font(20, "NSimSun", &mut prompt_font);
        nwg::Label::builder()
            .text("您是初次运行本程序，需要安装运行环境")
            .position((20, 20))
            .size((560, 20))
            .h_align(nwg::HTextAlign::Center)
            .font(Some(&prompt_font))
            .parent(&dialog.window)
            .build(&mut dialog.prompt_label1)?;
        nwg::Label::builder()
            .text("预计2-5分钟内完成，请耐心等待...")
            .position((20, 50))
            .size((560, 20))
            .h_align(nwg::HTextAlign::Center)
            .font(Some(&prompt_font))
            .parent(&dialog.window)
            .build(&mut dialog.prompt_label2)?;

        nwg::Label::builder()
            .text("正在初始化下载请求")
            .position((20, 90))
            .size((560, 20))
            .parent(&dialog.window)
            .build(&mut dialog.download_description_label)?;

        nwg::ProgressBar::builder()
            .flags(nwg::ProgressBarFlags::VISIBLE)
            .position((20, 112))
            .size((560, 32))
            .range(0..100)
            .pos(0)
            .parent(&dialog.window)
            .build(&mut dialog.download_progressbar)?;

        nwg::Label::builder()
            .text("正在等待下载完成...")
            .position((20, 150))
            .size((560, 20))
            .parent(&dialog.window)
            .build(&mut dialog.install_description_label)?;

        nwg::ProgressBar::builder()
            .flags(nwg::ProgressBarFlags::VISIBLE)
            .position((20, 172))
            .size((560, 32))
            .range(0..100)
            .pos(0)
            .marquee(false)
            .marquee_update(20)
            .parent(&dialog.window)
            .build(&mut dialog.install_progressbar)?;

        // Notice
        nwg::Notice::builder()
            .parent(&dialog.window)
            .build(&mut dialog.download_description_notice)?;

        nwg::Notice::builder()
            .parent(&dialog.window)
            .build(&mut dialog.download_rate_notice)?;

        nwg::Notice::builder()
            .parent(&dialog.window)
            .build(&mut dialog.install_description_notice)?;

        nwg::Notice::builder()
            .parent(&dialog.window)
            .build(&mut dialog.start_install_notice)?;

        nwg::Notice::builder()
            .parent(&dialog.window)
            .build(&mut dialog.install_done_notice)?;

        let ui = MainDlgUi {
            inner: Rc::new(dialog),
            default_handler: Default::default(),
        };

        let event_ui = Rc::downgrade(&ui.inner);
        let handle_events = move |event, _event_data, handle| {
            if let Some(dialog) = event_ui.upgrade() {
                match event {
                    E::OnWindowClose => {
                        if &handle == &dialog.window {
                            exit(2);
                        }
                    }
                    E::OnNotice => {
                        if &handle == &dialog.download_description_notice {
                            let t = { dialog.download_description.lock().unwrap() };
                            dialog.download_description_label.set_text(t.as_str());
                        } else if &handle == &dialog.download_rate_notice {
                            let r = { dialog.download_rate.lock().unwrap() };
                            dialog.download_progressbar.set_pos(*r as u32);
                        } else if &handle == &dialog.install_description_notice {
                            let t = { dialog.install_description.lock().unwrap() };
                            dialog.install_description_label.set_text(t.as_str());
                        } else if &handle == &dialog.start_install_notice {
                            let is_error = dialog.is_error.clone();
                            if { *is_error.lock().unwrap() } == true {
                                return;
                            }
                            dialog
                                .install_progressbar
                                .add_flags(nwg::ProgressBarFlags::MARQUEE);
                            dialog.install_progressbar.set_marquee(true, 20);
                            let start_install_depend =
                                { dialog.start_install_depend.lock().unwrap().clone() };
                            let install_description = dialog.install_description.clone();
                            let install_description_sender =
                                dialog.install_description_notice.sender();
                            let install_done_depend = dialog.install_done_depend.clone();
                            let install_done_sender = dialog.install_done_notice.sender();

                            let installing_depends_temp: Vec<String>;
                            {
                                let mut a = dialog.installing_depends.lock().unwrap();
                                a.push(start_install_depend.clone());
                                installing_depends_temp = a.clone();
                            }
                            {
                                *install_description.lock().unwrap() = format!(
                                    "正在安装运行环境：{}",
                                    installing_depends_temp.join(",")
                                );
                            }
                            install_description_sender.notice();

                            std::thread::spawn(move || {
                                match install_depend(start_install_depend.as_str()) {
                                    Ok(_) => {
                                        *install_done_depend.lock().unwrap() = start_install_depend;
                                        install_done_sender.notice();
                                    }
                                    Err(e) => {
                                        process_error(
                                            format!("安装出错：{}", e),
                                            true,
                                            true,
                                            true,
                                            false,
                                        );
                                        *install_done_depend.lock().unwrap() = start_install_depend;
                                        install_done_sender.notice();
                                    }
                                };
                            });
                        } else if &handle == &dialog.install_done_notice {
                            let install_done_depend =
                                { dialog.install_done_depend.lock().unwrap().clone() };
                            let install_description = dialog.install_description.clone();
                            let install_description_sender =
                                dialog.install_description_notice.sender();
                            let installing_depends_temp: Vec<String>;
                            let installed_depends_temp: Vec<String>;
                            {
                                let mut depends = dialog.installing_depends.lock().unwrap();
                                if let Some(pos) =
                                    depends.iter().position(|x| *x == install_done_depend)
                                {
                                    depends.remove(pos);
                                }
                                installing_depends_temp = depends.clone();
                            }
                            {
                                let mut depends = dialog.installed_depends.lock().unwrap();
                                depends.push(install_done_depend);
                                installed_depends_temp = depends.clone();
                            }
                            if installed_depends_temp.len() == dialog.needed_depends.len() {
                                dialog.close();
                            } else if installing_depends_temp.len() == 0 {
                                dialog.install_progressbar.set_pos(0);
                                dialog.install_progressbar.set_marquee(false, 20);
                                *install_description.lock().unwrap() =
                                    "正在等待下载完成...".to_string();
                                install_description_sender.notice();
                            } else {
                                *install_description.lock().unwrap() = format!(
                                    "正在安装运行环境：{}",
                                    installing_depends_temp.join("，")
                                );
                                install_description_sender.notice();
                            }
                        } else {
                            {
                                *dialog.is_error.lock().unwrap() = true;
                            }
                            process_error(
                                "notice handle不存在".to_string(),
                                true,
                                true,
                                true,
                                true,
                            );
                        }
                    }
                    E::OnInit => {
                        let needed_depends = dialog.needed_depends.clone();
                        let download_description = dialog.download_description.clone();
                        let download_description_sender =
                            dialog.download_description_notice.sender();
                        let download_rate = dialog.download_rate.clone();
                        let download_rate_sender = dialog.download_rate_notice.sender();
                        let start_install_depend = dialog.start_install_depend.clone();
                        let start_install_sender = dialog.start_install_notice.sender();
                        let is_error = dialog.is_error.clone();
                        std::thread::spawn(move || {
                            for (i, id) in needed_depends.iter().enumerate() {
                                if { *is_error.lock().unwrap() } == true {
                                    return;
                                }
                                {
                                    *download_description.lock().unwrap() = format!(
                                        "（{}/{}）正在下载运行环境：{}",
                                        i + 1,
                                        needed_depends.len(),
                                        id
                                    )
                                }
                                download_description_sender.notice();
                                let c_download_rate = download_rate.clone();
                                {
                                    *download_rate.lock().unwrap() = 0;
                                }
                                download_rate_sender.notice();

                                match download_depend(id, c_download_rate, download_rate_sender) {
                                    Ok(_) => {
                                        *start_install_depend.lock().unwrap() = id.to_string();
                                        start_install_sender.notice();
                                    }
                                    Err(e) => {
                                        {
                                            *is_error.lock().unwrap() = true;
                                        }
                                        process_error(
                                            format!("下载运行环境失败：{}", e),
                                            true,
                                            true,
                                            true,
                                            true,
                                        );
                                    }
                                };
                            }
                            *download_description.lock().unwrap() = format!(
                                "({}/{}) 所有下载已完成，请等待安装完成...",
                                needed_depends.len(),
                                needed_depends.len()
                            );
                            download_description_sender.notice();
                        });
                    }
                    _ => {}
                }
            }
        };

        *ui.default_handler.borrow_mut() = Some(nwg::full_bind_event_handler(
            &ui.inner.window.handle,
            handle_events,
        ));

        return Ok(ui);
    }
}

impl Drop for MainDlgUi {
    fn drop(&mut self) {
        let handler = self.default_handler.borrow();
        if handler.is_some() {
            nwg::unbind_event_handler(handler.as_ref().unwrap());
        }
    }
}

impl Deref for MainDlgUi {
    type Target = MainDlg;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
