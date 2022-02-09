use native_windows_gui as nwg;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::lib::error::process_error;
use crate::lib::process::exit;
use crate::lib::ui::try_build_font;
use crate::lib::update::{download_update, install_update};

#[derive(Default)]
pub struct UpdateDlg {
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

    resource_id: Arc<Mutex<i32>>,
}

impl UpdateDlg {
    pub fn set_resource_id(&mut self, resource_id: i32) {
        self.resource_id = Arc::new(Mutex::new(resource_id));
    }

    // fn close(&self) {
    //     nwg::stop_thread_dispatch();
    // }
}

pub struct UpdateDlgUi {
    inner: Rc<UpdateDlg>,
    default_handler: RefCell<Option<nwg::EventHandler>>,
}

impl nwg::NativeUi<UpdateDlgUi> for UpdateDlg {
    fn build_ui(mut dialog: Self) -> Result<UpdateDlgUi, nwg::NwgError> {
        use nwg::Event as E;

        let window_width = 500;
        let window_height = 190;

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
        try_build_font(17, "NSimSun", &mut prompt_font);
        nwg::Label::builder()
            .text("检测到有新版本，正在更新")
            .position((10, 10))
            .size((484, 18))
            .h_align(nwg::HTextAlign::Center)
            .font(Some(&prompt_font))
            .parent(&dialog.window)
            .build(&mut dialog.prompt_label1)?;
        nwg::Label::builder()
            .text("预计30秒内完成，请耐心等待...")
            .position((10, 30))
            .size((484, 18))
            .h_align(nwg::HTextAlign::Center)
            .font(Some(&prompt_font))
            .parent(&dialog.window)
            .build(&mut dialog.prompt_label2)?;

        nwg::Label::builder()
            .text("正在初始化下载请求")
            .position((10, 60))
            .size((484, 16))
            .parent(&dialog.window)
            .build(&mut dialog.download_description_label)?;

        nwg::ProgressBar::builder()
            .flags(nwg::ProgressBarFlags::VISIBLE)
            .position((10, 80))
            .size((484, 32))
            .range(0..100)
            .pos(0)
            .parent(&dialog.window)
            .build(&mut dialog.download_progressbar)?;

        nwg::Label::builder()
            .text("正在等待下载完成...")
            .position((10, 126))
            .size((484, 16))
            .parent(&dialog.window)
            .build(&mut dialog.install_description_label)?;

        nwg::ProgressBar::builder()
            .flags(nwg::ProgressBarFlags::VISIBLE)
            .position((10, 146))
            .size((484, 32))
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

        let ui = UpdateDlgUi {
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
                            dialog
                                .install_progressbar
                                .add_flags(nwg::ProgressBarFlags::MARQUEE);
                            dialog.install_progressbar.set_marquee(true, 20);
                        } else {
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
                        let download_description = dialog.download_description.clone();
                        let download_description_sender =
                            dialog.download_description_notice.sender();
                        let download_rate = dialog.download_rate.clone();
                        let download_rate_sender = dialog.download_rate_notice.sender();
                        let install_description = dialog.install_description.clone();
                        let install_description_sender = dialog.install_description_notice.sender();
                        let resource_id = dialog.resource_id.clone();
                        std::thread::spawn(move || {
                            {
                                *download_description.lock().unwrap() =
                                    "正在下载更新文件...".to_string()
                            }
                            download_description_sender.notice();
                            {
                                *download_rate.lock().unwrap() = 0;
                            }
                            download_rate_sender.notice();

                            match download_update(download_rate, download_rate_sender) {
                                Err(e) => {
                                    process_error(
                                        format!("下载更新文件失败\n{}", e),
                                        true,
                                        true,
                                        true,
                                        true,
                                    );
                                }
                                _ => {}
                            };
                            {
                                *download_description.lock().unwrap() = "下载已完成".to_string();
                            }
                            download_description_sender.notice();
                            {
                                *install_description.lock().unwrap() =
                                    "正在安装新版本...".to_string();
                            }
                            install_description_sender.notice();
                            let c_resource_id;
                            {
                                c_resource_id = (*resource_id).lock().unwrap().clone();
                            }
                            match install_update(c_resource_id) {
                                Err(e) => {
                                    process_error(
                                        format!("安装更新文件失败\n{}", e),
                                        true,
                                        true,
                                        true,
                                        true,
                                    );
                                }
                                _ => {}
                            };
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

impl Drop for UpdateDlgUi {
    fn drop(&mut self) {
        let handler = self.default_handler.borrow();
        if handler.is_some() {
            nwg::unbind_event_handler(handler.as_ref().unwrap());
        }
    }
}

impl Deref for UpdateDlgUi {
    type Target = UpdateDlg;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
