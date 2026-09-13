//! 客户端配置：站点、代理与认证信息的组合。
//!
//! [`EhClientConfig`] 是创建 [`EhClient`](crate::client::client::EhClient) 的输入，
//! 支持三种来源，可按优先级组合：
//!
//! 1. 代码内构造（[`EhClientConfig::default`] + 字段赋值）；
//! 2. YAML/JSON 配置文件反序列化（serde，字段名见各字段文档）；
//! 3. 环境变量（[`EhClientConfig::env`]，配合 `.env` 文件）。
//!
//! # 环境变量
//!
//! | 变量 | 含义 | 示例 |
//! |---|---|---|
//! | `EH_SITE` | 站点：`eh`/`e-hentai.org` 或 `ex`/`exhentai.org` | `eh` |
//! | `EH_PROXY` | 代理 URL | `socks5://127.0.0.1:7897` |
//! | `EH_AUTH_ID` / `EH_AUTH_HASH` / `EH_AUTH_IGNEOUS` | 认证 Cookie，见 [`EhClientAuth`] | — |
//!
//! # 示例
//!
//! ```rust
//! use libeh::client::config::EhClientConfig;
//!
//! // 从 YAML 反序列化（字段名与站点短代码见 dto::site）
//! let config: EhClientConfig = serde_yaml::from_str(
//!     "site: ex\nproxy:\n  protocol: http\n  host: 127.0.0.1\n  port: 7890\n",
//! )
//! .unwrap();
//! assert!(matches!(config.site, libeh::dto::site::Site::Ex));
//! ```

use std::env;

use serde::{Deserialize, Serialize};

use crate::dto::site::Site;
use crate::error::Error;

use super::{auth::EhClientAuth, proxy::EhClientProxy};

/// E-Hentai/ExHentai 客户端配置。
///
/// 序列化字段：`site`（`eh`/`ex`）、`proxy`（可空）、`auth`（可空），
/// 完整说明见[模块文档](self)。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EhClientConfig {
    /// 站点类型（YAML/JSON 短代码 `eh`/`ex`）。
    pub site: Site,
    /// 代理设置，默认为 `None`（直连）。
    pub proxy: Option<EhClientProxy>,
    /// 用户身份验证设置，默认为 `None`（匿名访问）。
    pub auth: Option<EhClientAuth>,
}

impl EhClientConfig {
    /// 解析 `EH_SITE` 环境变量为站点类型。
    ///
    /// 接受站点短代码（`eh`/`ex`）与完整域名两种写法；
    /// 变量未设置时默认表站。值无法识别时返回 [`Error::Config`]，
    /// 避免 [`Site::Un`](crate::dto::site::Site::Un) 在后续构建 URL 时才暴露问题。
    fn site_from_env() -> Result<Site, Error> {
        match env::var("EH_SITE") {
            Err(_) => Ok(Site::Eh),
            Ok(raw) => match raw.trim().to_ascii_lowercase().as_str() {
                "" | "eh" | "e-hentai.org" => Ok(Site::Eh),
                "ex" | "exhentai.org" => Ok(Site::Ex),
                other => Err(Error::Config(format!(
                    "unsupported EH_SITE value {other:?}; expected \"eh\", \"ex\", \
                     \"e-hentai.org\" or \"exhentai.org\""
                ))),
            },
        }
    }

    /// 从环境变量读取配置，见[模块文档](self)#环境变量。
    ///
    /// # Errors
    ///
    /// `EH_SITE` 的值无法识别时返回 [`Error::Config`]。
    /// 代理与认证信息缺失时对应字段为 `None`，不视为错误。
    pub fn env() -> Result<Self, Error> {
        Ok(EhClientConfig {
            site: Self::site_from_env()?,
            proxy: EhClientProxy::env(),
            auth: EhClientAuth::env(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::client::{config::EhClientConfig, proxy::EhClientProxy};

    #[test]
    fn config_to_json() {
        let config = EhClientConfig {
            proxy: Some(EhClientProxy::new("http", "127.0.0.1", 7890)),
            ..EhClientConfig::default()
        };
        let result = serde_json::to_string(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn config_to_yaml() {
        let config = EhClientConfig {
            proxy: Some(EhClientProxy::new("http", "127.0.0.1", 7890)),
            auth: Some(crate::client::auth::EhClientAuth::new("1", "2", None)),
            ..EhClientConfig::default()
        };
        let result = serde_yaml::to_string(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn config_from_json() {
        let config = r#"
{
    "site": "ex",
    "proxy": {
        "protocol": "http",
        "host": "127.0.0.1",
        "port": 7890
    }
}
"#;
        let result = serde_json::from_str::<EhClientConfig>(config);
        assert!(result.is_ok());
    }

    #[test]
    fn config_from_yaml() {
        let config = r#"
site: ex
proxy:
  protocol: http
  host: 127.0.0.1
  port: 7890
"#;
        let result = serde_yaml::from_str::<EhClientConfig>(config);
        assert!(result.is_ok());
    }
}
