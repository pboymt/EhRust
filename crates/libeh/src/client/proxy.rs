use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::env;

/// EhClient 代理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EhClientProxy {
    /// 代理协议
    pub protocol: String,
    /// 代理主机
    pub host: String,
    /// 代理端口
    pub port: i32,
}

impl EhClientProxy {
    /// 创建一个新的 EhClientProxy 实例
    pub fn new(protocol: &str, host: &str, port: i32) -> Self {
        EhClientProxy {
            protocol: protocol.to_string(),
            host: host.to_string(),
            port,
        }
    }

    /// 从环境变量中获取代理设置
    pub fn env() -> Option<Self> {
        let url = if let Ok(url) = env::var("EH_PROXY") {
            url
        } else if let Ok(url) = env::var("HTTP_PROXY") {
            url
        } else {
            return None;
        };
        if let Ok(url) = Url::parse(&url) {
            let schema = url.scheme();
            let host = url.host_str().unwrap_or("localhost");
            let port = url.port().unwrap_or(1080);
            Some(EhClientProxy::new(schema, host, port.into()))
        } else {
            None
        }
    }
}

impl ToString for EhClientProxy {
    /// 将 EhClientProxy 转换为 URL 字符串
    fn to_string(&self) -> String {
        format!("{}://{}:{}", self.protocol, self.host, self.port)
    }
}
