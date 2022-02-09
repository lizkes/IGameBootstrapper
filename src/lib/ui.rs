use native_windows_gui as nwg;

use crate::lib::error::process_error;

pub fn try_build_font(size: u32, family: &str, font: &mut nwg::Font) {
    let result = nwg::Font::builder()
        .size(size)
        .family(family)
        .weight(0)
        .build(font);
    if result.is_err() {
        match nwg::Font::builder()
            .size(size)
            .family("Arial")
            .weight(0)
            .build(font)
        {
            Ok(_) => {}
            Err(e) => process_error(format!("构建Arial字体失败：{}", e), true, true, true, true),
        };
    }
}
