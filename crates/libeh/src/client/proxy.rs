//! 代理配置。
//!
//! [`EhClientProxy`] 描述一个正向代理，支持 `http`/`https`/`socks5` 协议
//! （`socks5` 依赖 reqwest 的 `socks` feature，本库已启用）。
//!
//! # 环境变量
//!
//! [`EhClientProxy::env`] 读取 `EH_PROXY`（本库专用，优先），
//! 值形如 `socks5://127.0.0.1:7897`。
//!
//! 标准的 `HTTP_PROXY`/`HTTPS_PROXY`/`ALL_PROXY` 环境变量**不在此处理**：
//! reqwest 客户端在未显式设置代理时会自动遵循这些标准变量，
//! 这里读取它们反而会覆盖并丢失 `HTTPS_PROXY` 的语义。
//!
//! # 示例
//!
//! ```rust
//! use libeh::client::proxy::EhClientProxy;
//!
//! let proxy = EhClientProxy::new("socks5", "127.0.0.1", 7897);
//! assert_eq!(proxy.to_string(), "socks5://127.0.0.1:7897");
//! ```

use std::env;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::Error;

/// 代理连接支持的协议。
const SUPPORTED_PROTOCOLS: [&str; 3] = ["http", "https", "socks5"];

/// 代理设置（协议 + 主机 + 端口）。
///
/// 序列化形式与配置文件中的 `proxy:` 段对应，详见[模块文档](self)。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EhClientProxy {
    /// 代理协议：`http` / `https` / `socks5`。
    pub protocol: String,
    /// 代理主机（IP 或域名）。
    pub host: String,
    /// 代理端口。
    pub port: u16,
}

impl EhClientProxy {
    /// 创建一个新的 [`EhClientProxy`] 实例。
    ///
    /// `protocol` 应为 `http`/`https`/`socks5` 之一；
    /// 完整校验在 [`EhClient::try_new`](crate::client::client::EhClient::try_new) 中进行。
    #[must_use]
    pub fn new(protocol: &str, host: &str, port: u16) -> Self {
        EhClientProxy {
            protocol: protocol.to_string(),
            host: host.to_string(),
            port,
        }
    }

    /// 从环境变量 `EH_PROXY` 读取代理设置，见[模块文档](self)。
    ///
    /// 变量未设置或无法解析为受支持的代理 URL 时返回 `None`
    /// （不报错——代理是可选配置）。
    #[must_use]
    pub fn env() -> Option<Self> {
        let url = env::var("EH_PROXY").ok()?;
        Self::from_url(&url).ok()
    }

    /// 解析代理 URL（如 `socks5://127.0.0.1:7897`）。
    ///
    /// # Errors
    ///
    /// URL 无法解析、缺少端口或协议不受支持时返回 [`Error::Config`]。
    pub fn from_url(url: &str) -> Result<Self, Error> {
        let parsed = reqwest::Url::parse(url)
            .map_err(|e| Error::Config(format!("invalid proxy url {url:?}: {e}")))?;
        let scheme = parsed.scheme().to_ascii_lowercase();
        if !SUPPORTED_PROTOCOLS.contains(&scheme.as_str()) {
            return Err(Error::Config(format!(
                "unsupported proxy protocol {scheme:?}; expected one of {SUPPORTED_PROTOCOLS:?}"
            )));
        }
        let Some(host) = parsed.host_str() else {
            return Err(Error::Config(format!("proxy url {url:?} has no host")));
        };
        let Some(port) = parsed.port() else {
            return Err(Error::Config(format!("proxy url {url:?} has no port")));
        };
        Ok(EhClientProxy {
            protocol: scheme,
            host: host.to_string(),
            port,
        })
    }

    /// 转换为 reqwest 的代理 URL 字符串（`protocol://host:port`）。
    #[must_use]
    fn to_proxy_url(&self) -> String {
        format!("{}://{}:{}", self.protocol, self.host, self.port)
    }
}

impl fmt::Display for EhClientProxy {
    /// 输出为代理 URL 字符串，如 `socks5://127.0.0.1:7897`。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_proxy_url())
    }
}
