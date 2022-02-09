use crate::lib::error::process_error;
use crate::lib::time::utc_str_to_china_str;
use serde::Deserialize;
use std::fmt;

use crate::static_var;

pub enum ProviderGroup {
    Fast,
    #[allow(dead_code)]
    Normal,
}

impl fmt::Display for ProviderGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderGroup::Fast => write!(f, "fast"),
            ProviderGroup::Normal => write!(f, "normal"),
        }
    }
}

fn hand_api_response(e: ureq::Error) {
    #[derive(Deserialize)]
    struct ErrorResp {
        code: u32,
        content: String,
    }

    match e {
        ureq::Error::Status(c, r) => match c {
            400 => {
                let error_response_json: ErrorResp = r
                    .into_json()
                    .map_err(|e| {
                        process_error(
                            format!("反序列化错误响应失败\n{:?}", e),
                            true,
                            true,
                            true,
                            true,
                        );
                    })
                    .unwrap();
                process_error(
                    format!(
                        "客户端发送请求错误，状态码400，请稍后再试\n{}",
                        error_response_json.content
                    ),
                    true,
                    true,
                    true,
                    true,
                );
            }
            500 => {
                let error_response_json: ErrorResp = r
                    .into_json()
                    .map_err(|e| {
                        process_error(
                            format!("反序列化错误响应失败\n{:?}", e),
                            true,
                            true,
                            true,
                            true,
                        );
                    })
                    .unwrap();
                if error_response_json.code == 500 {
                    process_error(
                        format!(
                            "服务器维护中\n预计将于{:?}恢复正常",
                            utc_str_to_china_str(error_response_json.content.as_str())
                        ),
                        true,
                        true,
                        false,
                        true,
                    );
                } else {
                    process_error(
                        format!(
                            "服务器响应错误，状态码500，请稍后再试\n{}",
                            error_response_json.content
                        ),
                        true,
                        true,
                        true,
                        true,
                    );
                }
            }
            _ => {
                process_error(
                    format!("服务器响应错误，状态码，请稍后再试{}\n{:?}", c, r),
                    true,
                    true,
                    true,
                    true,
                );
            }
        },
        ureq::Error::Transport(t) => {
            process_error(
                format!("从服务器获取响应失败\n{:?}", t),
                true,
                true,
                true,
                true,
            );
        }
    }
}

pub fn get_download_url(resource_id: i32, provider_group: &ProviderGroup) -> String {
    #[derive(Deserialize)]
    struct DownloadUrlResp {
        download_url: String,
    }

    let request_url = format!(
        "https://api.igame.ml/resource/{}/download_url?provider_group={}",
        resource_id, provider_group
    );
    let response = (*static_var::UREQ_AGENT)
        .get(request_url.as_str())
        .call()
        .map_err(|e| hand_api_response(e))
        .unwrap();
    let response_json: DownloadUrlResp = response
        .into_json()
        .map_err(|e| {
            process_error(
                format!("反序列化响应失败：{}\n{:?}", request_url, e),
                true,
                true,
                true,
                true,
            );
        })
        .unwrap();

    return response_json.download_url;
}

pub fn get_resourc_version(resource_id: i32) -> String {
    #[derive(Deserialize)]
    struct ResourceVersionResp {
        version: String,
    }

    let request_url = format!("https://api.igame.ml/resource/{}/version", resource_id);
    let response = (*static_var::UREQ_AGENT)
        .get(request_url.as_str())
        .call()
        .map_err(|e| hand_api_response(e))
        .unwrap();
    let response_json: ResourceVersionResp = response
        .into_json()
        .map_err(|e| {
            process_error(
                format!("反序列化响应失败：{}\n{:?}", request_url, e),
                true,
                true,
                true,
                true,
            );
        })
        .unwrap();

    return response_json.version;
}
