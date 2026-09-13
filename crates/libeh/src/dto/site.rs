//! 站点类型枚举：e-hentai（表站）与 exhentai（里站）。
//!
//! 表站无需登录即可浏览大部分内容；里站要求请求携带有效的
//! `ipb_member_id`/`ipb_pass_hash`/`igneous` Cookie，否则返回
//! sad panda 引导页（在 [`crate::error::Error::Protocol`] 中体现）。
//!
//! # 示例
//!
//! ```rust
//! use libeh::dto::site::Site;
//!
//! let site = Site::from("https://e-hentai.org/g/1/abcd123456/".to_string());
//! assert_eq!(site, Site::Eh);
//! assert_eq!(site.url().unwrap().as_str(), "https://e-hentai.org/");
//! ```

use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::error::Error;

/// 站点类型。
///
/// 序列化形式为短代码（`un`/`eh`/`ex`），与 YAML/JSON 配置文件中的写法一致。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Site {
    /// 未知站点：输入既不是 e-hentai.org 也不是 exhentai.org。
    #[serde(rename = "un")]
    Un,
    /// e-hentai.org（表站）。
    #[serde(rename = "eh")]
    Eh,
    /// exhentai.org（里站，需要认证 Cookie）。
    #[serde(rename = "ex")]
    Ex,
}

impl Default for Site {
    /// 默认站点为表站（[`Site::Eh`]）：无需登录即可访问。
    fn default() -> Self {
        Site::Eh
    }
}

impl Site {
    /// 依据裸域名（如 `"e-hentai.org"`）识别站点，无法识别返回 [`Site::Un`]。
    #[must_use]
    pub fn from_domain(domain: &str) -> Site {
        match domain {
            "e-hentai.org" | "www.e-hentai.org" | "api.e-hentai.org" => Site::Eh,
            "exhentai.org" | "www.exhentai.org" | "api.exhentai.org" => Site::Ex,
            _ => Site::Un,
        }
    }

    /// 把站点转换为对应根 URL。
    ///
    /// # Errors
    ///
    /// [`Site::Un`] 没有对应站点，返回 [`Error::Config`]。
    /// （历史版本中此转换会直接 panic。）
    pub fn url(self) -> Result<Url, Error> {
        match self {
            Site::Eh => Ok(Url::parse("https://e-hentai.org/").expect("constant url")),
            Site::Ex => Ok(Url::parse("https://exhentai.org/").expect("constant url")),
            Site::Un => Err(Error::Config(
                "site is not set (Site::Un); expected e-hentai.org or exhentai.org".into(),
            )),
        }
    }
}

impl From<String> for Site {
    /// 依据域名片段识别站点（子串匹配即可，如完整画廊 URL）。
    ///
    /// 无法识别时返回 [`Site::Un`] 而不是 panic。
    fn from(value: String) -> Self {
        if value.contains("e-hentai.org") {
            Site::Eh
        } else if value.contains("exhentai.org") {
            Site::Ex
        } else {
            Site::Un
        }
    }
}

impl From<Site> for Url {
    /// # Panics
    ///
    /// [`Site::Un`] 没有对应 URL，会 panic。
    /// 新代码请使用返回 `Result` 的 [`Site::url`]。
    fn from(value: Site) -> Self {
        value
            .url()
            .expect("Site::Un has no url; use Site::url() instead")
    }
}
