use rand::Rng;
use std::fs::{self, copy, create_dir_all, remove_dir_all, remove_file, rename};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use crate::library::error::process_error;
use crate::static_var;

// pub fn path_to_string(path: &PathBuf) -> String {
//     return path.clone().into_os_string().into_string().unwrap();
// }

pub fn get_temp_file_path(name: &str) -> PathBuf {
    let mut file_path_buf = std::env::temp_dir();
    file_path_buf.push(name);
    return file_path_buf;
}

pub fn get_random_temp_dir_path() -> PathBuf {
    let mut dir_path_buf = std::env::temp_dir();
    let random_string: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    dir_path_buf.push(random_string);
    while dir_path_buf.exists() {
        let random_string: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        dir_path_buf.pop();
        dir_path_buf.push(random_string);
    }
    return dir_path_buf;
}

// pub fn read_temp_file(name: &str) -> Result<(PathBuf, fs::File), std::io::Error> {
//     let mut temp_path = std::env::temp_dir();
//     temp_path.push(name);
//     let temp_file = fs::File::options().read(true).open(temp_path.clone())?;
//     return Ok((temp_path, temp_file));
// }

pub fn write_temp_file(name: &str, append: bool) -> Result<(PathBuf, fs::File), std::io::Error> {
    let mut temp_path = std::env::temp_dir();
    temp_path.push(name);
    let temp_file = fs::File::options()
        .write(true)
        .truncate(!append)
        .append(append)
        .create(true)
        .open(temp_path.clone())?;
    return Ok((temp_path, temp_file));
}

pub fn try_search_resource_id() -> i32 {
    match search_resource_id() {
        Ok(v) => v,
        Err(e) => {
            process_error(format!("检索自身信息失败：{}", e), true, true, true, true);
            return 0;
        }
    }
}

fn byte_search(src: &[u8], pattern: &[u8]) -> Option<usize> {
    let max_first_char_slot = src.len() - pattern.len() + 1;
    for i in 0..max_first_char_slot {
        if src[i] != pattern[0] {
            continue;
        }

        for j in (1..pattern.len()).rev() {
            if src[i + j] != pattern[j] {
                break;
            }
            if j == 1 {
                return Some(i);
            }
        }
    }

    return None;
}

fn search_resource_id() -> Result<i32, io::Error> {
    const MAGIC_BYTES: [u8; 16] = [
        0xea, 0x7f, 0xd6, 0x96, 0x1a, 0x08, 0x71, 0xd3, 0xc1, 0x44, 0x7c, 0x8b, 0x1b, 0xb0, 0xa3,
        0x36,
    ];
    let self_file = fs::File::open((*static_var::CURRENT_EXE_PATH).clone())?;
    let self_file_size = self_file.metadata()?.len();
    let mut reader = io::BufReader::new(self_file);
    reader.seek(SeekFrom::Start(self_file_size - 65536))?;
    let mut total_read: u64 = self_file_size - 65536;
    let mut buffer: [u8; 8192] = [0; 8192];
    let mut position: Option<u64> = None;

    loop {
        let read_count = reader.read(&mut buffer)?;
        if read_count < 16 {
            break;
        } else {
            let magic_bytes_position = byte_search(&buffer[..read_count], &MAGIC_BYTES);
            if magic_bytes_position.is_none() {
                //未找到，回退15byte
                reader.seek_relative(-15)?;
                total_read += read_count as u64 - 15;
                continue;
            } else {
                total_read = total_read + magic_bytes_position.unwrap() as u64 + 16;
                position = Some(total_read);
                break;
            }
        }
    }

    if position.is_some() {
        reader.seek(SeekFrom::Start(position.unwrap()))?;
        let mut buffer: [u8; 4] = [0; 4];
        reader.read(&mut buffer)?;
        let id = i32::from_be_bytes(buffer);
        return Ok(id);
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "检索资源ID失败",
        ));
    }
}

pub fn try_copy_file(src_path: &PathBuf, dst_path: &PathBuf) -> Result<(), String> {
    if src_path.exists() && src_path.is_file() {
        if dst_path.exists() {
            try_remove_path(dst_path)?;
        }

        match copy(src_path, dst_path) {
            Err(e) => {
                return Err(format!(
                    "复制文件失败：{:?} -> {:?}\n{:?}",
                    src_path, dst_path, e
                ));
            }
            _ => {}
        };
    }

    return Ok(());
}

pub fn try_move_file(src_path: &PathBuf, dst_path: &PathBuf) -> Result<(), String> {
    if src_path.exists() && src_path.is_file() {
        if dst_path.exists() {
            try_remove_path(dst_path)?;
        }

        match rename(src_path, dst_path) {
            Err(e) => {
                return Err(format!(
                    "移动文件失败：{:?} -> {:?}\n{:?}",
                    src_path, dst_path, e
                ));
            }
            _ => {}
        };
    }

    return Ok(());
}

fn retry_remove_file(path: &PathBuf) -> io::Result<()> {
    for _ in 1..20 {
        match remove_file(path) {
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            _ => return Ok(()),
        }
    }
    remove_file(path)
}

fn retry_remove_dir_all(path: &PathBuf) -> io::Result<()> {
    for _ in 1..20 {
        match remove_dir_all(path) {
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            _ => return Ok(()),
        }
    }
    remove_dir_all(path)
}

pub fn try_remove_path(path: &PathBuf) -> Result<(), String> {
    if path.exists() {
        if path.is_file() {
            match retry_remove_file(path) {
                Err(e) => {
                    return Err(format!("删除文件失败：{:?}\n{:?}", path, e));
                }
                _ => {}
            };
        } else if path.is_dir() {
            match retry_remove_dir_all(path) {
                Err(e) => {
                    return Err(format!("删除文件夹失败：{:?}\n{:?}", path, e));
                }
                _ => {}
            };
        }
    }

    return Ok(());
}

pub fn extract_tzst(tzst_path: &PathBuf, dir_path: &PathBuf) -> Result<(), String> {
    match create_dir_all(dir_path) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!("创建文件夹失败：{:?}\n{:?}", dir_path, e));
        }
    };

    let tzst_file = match fs::File::options().read(true).open(tzst_path) {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("打开tzst文件失败：{:?}\n{:?}", tzst_path, e));
        }
    };
    let zstd_reader = match zstd::stream::Decoder::new(tzst_file) {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("zstd解压缩文件失败\n{:?}", e));
        }
    };
    let mut archive = tar::Archive::new(zstd_reader);
    match archive.unpack(dir_path) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!("tar解压缩文件失败\n{:?}", e));
        }
    };
    drop(archive);

    return Ok(());
}

pub fn write_resource_id_to_file(path: &PathBuf, resource_id: i32) -> Result<(), String> {
    const MAGIC_BYTES: [u8; 8] = [0x77, 0x77, 0x77, 0x77, 0xFF, 0xFF, 0xFF, 0xFF];

    let mut file = match fs::File::options()
        .write(true)
        .truncate(false)
        .append(true)
        .create(true)
        .open(path)
    {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("获取文件描述符失败：{:?}\n{:?}", path, e));
        }
    };

    let mut buffer = MAGIC_BYTES.to_vec();
    buffer.extend_from_slice(&resource_id.to_be_bytes());
    match file.write_all(&buffer) {
        Err(e) => {
            return Err(format!("写入文件失败：{:?}\n{:?}", path, e));
        }
        _ => {}
    };

    return Ok(());
}
