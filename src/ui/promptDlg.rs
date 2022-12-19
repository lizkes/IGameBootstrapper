use native_windows_gui as nwg;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::library::process::exit;
use crate::library::ui::try_build_font;

#[derive(Default)]
pub struct PromptDlg {
    window: nwg::Window,
    window_icon: nwg::Icon,
    embed_resource: nwg::EmbedResource,

    prompt_label1: nwg::Label,
    prompt_label2: nwg::Label,
    prompt_label3: nwg::Label,
    prompt_label4: nwg::Label,
    prompt_label5: nwg::Label,
    prompt_label6: nwg::Label,
    confirm_button: nwg::Button,

    countdown_notice: nwg::Notice,
    countdown_value: Arc<Mutex<i8>>,

    needed_depends: Vec<String>,
}

impl PromptDlg {
    pub fn set_needed_depends(&mut self, depends: &Vec<&str>) {
        self.needed_depends = depends.iter().map(|s| s.to_string()).collect();
    }

    fn close(&self) {
        nwg::stop_thread_dispatch();
    }
}

pub struct PromptDlgUi {
    inner: Rc<PromptDlg>,
    default_handler: RefCell<Option<nwg::EventHandler>>,
}

impl nwg::NativeUi<PromptDlgUi> for PromptDlg {
    fn build_ui(mut dialog: Self) -> Result<PromptDlgUi, nwg::NwgError> {
        use nwg::Event as E;

        let window_width = 600;
        let window_height = 270;

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
            .title("运行前提示")
            .center(true)
            .icon(Some(&dialog.window_icon))
            .build(&mut dialog.window)?;

        let mut prompt_font = nwg::Font::default();
        try_build_font(18, "NSimSun", &mut prompt_font);
        nwg::Label::builder()
            .text("为了启动IGame安装器，我们需要安装额外的运行环境")
            .position((20, 20))
            .size((560, 20))
            .h_align(nwg::HTextAlign::Center)
            .font(Some(&prompt_font))
            .parent(&dialog.window)
            .build(&mut dialog.prompt_label1)?;
        nwg::Label::builder()
            .text(&format!(
                "将要安装的运行环境有：{}",
                dialog.needed_depends.join("，")
            ))
            .position((20, 50))
            .size((560, 20))
            .h_align(nwg::HTextAlign::Center)
            .font(Some(&prompt_font))
            .parent(&dialog.window)
            .build(&mut dialog.prompt_label6)?;
        nwg::Label::builder()
            .text("在安装过程中，您电脑上的杀毒软件可能会弹出窗口阻止安装")
            .position((20, 80))
            .size((560, 20))
            .h_align(nwg::HTextAlign::Center)
            .font(Some(&prompt_font))
            .parent(&dialog.window)
            .build(&mut dialog.prompt_label2)?;
        nwg::Label::builder()
            .text("为了可以正确安装，请在弹出窗口的时候点击允许")
            .position((20, 110))
            .size((560, 20))
            .h_align(nwg::HTextAlign::Center)
            .font(Some(&prompt_font))
            .parent(&dialog.window)
            .build(&mut dialog.prompt_label3)?;
        nwg::Label::builder()
            .text("本软件保证纯净无毒，不会进行一切与安装运行环境无关的行为")
            .position((20, 140))
            .size((560, 20))
            .h_align(nwg::HTextAlign::Center)
            .font(Some(&prompt_font))
            .parent(&dialog.window)
            .build(&mut dialog.prompt_label4)?;
        nwg::Label::builder()
            .text("如果您不信任我，请点击右上角关闭本软件")
            .position((20, 170))
            .size((560, 20))
            .h_align(nwg::HTextAlign::Center)
            .font(Some(&prompt_font))
            .parent(&dialog.window)
            .build(&mut dialog.prompt_label5)?;

        let mut button_font = nwg::Font::default();
        try_build_font(20, "NSimSun", &mut button_font);
        nwg::Button::builder()
            .enabled(false)
            .focus(false)
            .text("请仔细阅读上方的提示 5秒")
            .position((150, 210))
            .size((300, 40))
            .font(Some(&button_font))
            .parent(&dialog.window)
            .build(&mut dialog.confirm_button)?;

        // Notice
        nwg::Notice::builder()
            .parent(&dialog.window)
            .build(&mut dialog.countdown_notice)?;

        let ui = PromptDlgUi {
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
                    E::OnButtonClick => {
                        if &handle == &dialog.confirm_button {
                            dialog.close();
                        }
                    }
                    E::OnNotice => {
                        if &handle == &dialog.countdown_notice {
                            let v = { dialog.countdown_value.lock().unwrap() };
                            if *v != 0 {
                                dialog
                                    .confirm_button
                                    .set_text(&format!("请仔细阅读上方的提示 {}秒", *v));
                            } else {
                                dialog.confirm_button.set_text("我了解并已经关闭杀毒软件");
                                dialog.confirm_button.set_enabled(true);
                                dialog.confirm_button.set_focus();
                            }
                        }
                    }
                    E::OnInit => {
                        let confirm_button = dialog.countdown_value.clone();
                        let countdown_sender = dialog.countdown_notice.sender();
                        std::thread::spawn(move || {
                            for i in (0..5).rev() {
                                std::thread::sleep(std::time::Duration::from_secs(1));
                                *confirm_button.lock().unwrap() = i;
                                countdown_sender.notice();
                            }
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

impl Drop for PromptDlgUi {
    fn drop(&mut self) {
        let handler = self.default_handler.borrow();
        if handler.is_some() {
            nwg::unbind_event_handler(handler.as_ref().unwrap());
        }
    }
}

impl Deref for PromptDlgUi {
    type Target = PromptDlg;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
