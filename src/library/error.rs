use std::io::Write;

use crate::library::crypto::encrypt_message;
use crate::library::file::write_temp_file;
use crate::library::time::generate_timestamp;
use crate::library::window::open_error_message_box;

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
        crate::library::process::exit(1);
    }
}
