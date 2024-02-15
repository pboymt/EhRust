use serde::{Deserialize, Serialize};

/// EhClient 代理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EhClientProxy {
    pub protocol: String,
    pub host: String,
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
}

impl ToString for EhClientProxy {
    /// 将 EhClientProxy 转换为 URL 字符串
    fn to_string(&self) -> String {
        format!("{}://{}:{}", self.protocol, self.host, self.port)
    }
}
