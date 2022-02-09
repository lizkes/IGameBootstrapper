use native_windows_gui as nwg;
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};

use crate::lib::file::write_temp_file;

pub fn download_file(
    url: &str,
    file_name: &str,
    rate: Arc<Mutex<u8>>,
    rate_sender: nwg::NoticeSender,
) -> Result<(), String> {
    let agent = ureq::builder()
        .timeout_connect(std::time::Duration::from_secs(10))
        .build();
    let resp = match agent.get(url).call() {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("发送请求失败\n{:?}", e));
        }
    };
    let total_size = match resp.header("content-length") {
        Some(v) => v.parse::<usize>().unwrap_or(52428800),
        None => 52428800,
    };
    let resp_reader = resp.into_reader();
    let mut reader = io::BufReader::new(resp_reader);

    let (_, fd) = match write_temp_file(file_name, false) {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("创建文件失败\n{:?}", e));
        }
    };
    let mut writer = io::BufWriter::new(fd);

    let mut download_size: usize = 0;
    let mut read_size: usize;
    let mut buffer: [u8; 8192] = [0; 8192];
    read_size = match reader.read(&mut buffer) {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("获取下载文件失败\n{:?}", e));
        }
    };

    while read_size != 0 {
        let write_size = match writer.write(&buffer[0..read_size]) {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("写入下载文件失败\n{:?}", e));
            }
        };
        download_size += write_size;
        let download_rate = (download_size as f64 / total_size as f64 * 100.0).floor() as u8;
        {
            let mut r = rate.lock().unwrap();
            if *r != download_rate {
                *r = download_rate;
                rate_sender.notice();
            }
        }
        read_size = match reader.read(&mut buffer) {
            Ok(v) => v as usize,
            Err(e) => {
                return Err(format!("获取下载文件失败\n{:?}", e));
            }
        };
    }

    return Ok(());
}

pub fn download_file_without_notice(url: &str, file_name: &str) -> Result<(), String> {
    let agent = ureq::builder()
        .timeout_connect(std::time::Duration::from_secs(10))
        .build();
    let resp = match agent.get(url).call() {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("发送请求失败\n{:?}", e));
        }
    };
    let resp_reader = resp.into_reader();
    let mut reader = io::BufReader::new(resp_reader);

    let (_, fd) = match write_temp_file(file_name, false) {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("创建文件失败\n{:?}", e));
        }
    };
    let mut writer = io::BufWriter::new(fd);

    let mut read_size: usize;
    let mut buffer: [u8; 8192] = [0; 8192];

    read_size = match reader.read(&mut buffer) {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("获取下载文件失败\n{:?}", e));
        }
    };
    while read_size != 0 {
        match writer.write_all(&buffer[0..read_size]) {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("写入下载文件失败\n{:?}", e));
            }
        };
        read_size = match reader.read(&mut buffer) {
            Ok(v) => v as usize,
            Err(e) => {
                return Err(format!("获取下载文件失败\n{:?}", e));
            }
        };
    }

    return Ok(());
}
