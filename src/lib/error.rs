use std::io::Write;

use crate::lib::crypto::encrypt_message;
use crate::lib::file::write_temp_file;
use crate::lib::time::generate_timestamp;
use crate::lib::window::open_error_message_box;

pub fn process_error(message: String, open_box: bool, write_log: bool, upload: bool, exit: bool) {
    if open_box {
        open_error_message_box(message.as_str());
    }
    if write_log {
        let (_, mut error_file) = write_temp_file("IGameBootstrapError.log", true).unwrap();
        error_file
            .write_all(format!("{}\n{}\n", generate_timestamp(), message).as_bytes())
            .unwrap();
    }
    if upload {
        ureq::post("https://api.igame.ml/error/collect")
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(10))
            .send_json(ureq::json!({
                "app_name": "IGameBootstrapper",
                "app_version": env!("CARGO_PKG_VERSION"),
                "content": encrypt_message(message.as_str())
            }))
            .unwrap();
    }
    if exit {
        crate::lib::process::exit(1);
    }
}
