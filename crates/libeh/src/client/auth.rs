//! 用户身份验证信息（认证 Cookie 三件套）。
//!
//! E-Hentai/ExHentai 的登录态**完全由 Cookie 承载**，没有独立的 API token：
//!
//! | Cookie | 含义 | 必需性 |
//! |---|---|---|
//! | `ipb_member_id` | 论坛（IPB）会员 ID | 表站/里站登录必需 |
//! | `ipb_pass_hash` | 会员密码哈希 | 表站/里站登录必需 |
//! | `igneous` | 里站准入凭证 | 仅访问 exhentai.org 需要 |
//!
//! [`EhClientAuth`] 由 [`EhClient`](crate::client::client::EhClient) 注入
//! HTTP CookieJar；`Debug` 实现对 `ipb_pass_hash` 与 `igneous` 做了脱敏，
//! 可以放心打印/记录。
//!
//! # 环境变量
//!
//! [`EhClientAuth::env`] 读取以下变量（配合 `.env` 文件使用）：
//!
//! - `EH_AUTH_ID`：`ipb_member_id`
//! - `EH_AUTH_HASH`：`ipb_pass_hash`
//! - `EH_AUTH_IGNEOUS`：`igneous`（可选；缺省表示无里站权限）
//!
//! 三者均未设置时返回 `None`（匿名访问）。

use std::env;
use std::fmt;

use serde::{Deserialize, Serialize};

/// E-Hentai/ExHentai 用户身份验证信息。
///
/// 字段值即对应 Cookie 的原值，详见[模块文档](self)。
/// `Debug` 输出对敏感字段脱敏（如 `ipb_pass_hash: "****"`）。
#[derive(Clone, Serialize, Deserialize)]
pub struct EhClientAuth {
    /// E-Hentai/ExHentai 用户 ID（Cookie `ipb_member_id`）。
    pub ipb_member_id: String,
    /// E-Hentai/ExHentai 用户令牌（Cookie `ipb_pass_hash`）。
    pub ipb_pass_hash: String,
    /// ExHentai 访问令牌（Cookie `igneous`），为 `None` 时无 ExHentai 访问权限。
    pub igneous: Option<String>,
}

impl EhClientAuth {
    /// 创建一个新的 [`EhClientAuth`] 实例。
    ///
    /// 字段不做格式校验（站点以实际响应为准）；
    /// `igneous` 传 `Some("")` 与 `None` 等价于不携带该 Cookie。
    #[must_use]
    pub fn new(ipb_member_id: &str, ipb_pass_hash: &str, igneous: Option<&str>) -> Self {
        EhClientAuth {
            ipb_member_id: ipb_member_id.to_string(),
            ipb_pass_hash: ipb_pass_hash.to_string(),
            igneous: igneous
                .filter(|s| !s.is_empty())
                .map(std::string::ToString::to_string),
        }
    }

    /// 从环境变量读取认证信息，见[模块文档](self)#环境变量。
    ///
    /// `EH_AUTH_ID` 或 `EH_AUTH_HASH` 缺失时返回 `None`（匿名访问），
    /// 不会因为配置不完整而报错。
    #[must_use]
    pub fn env() -> Option<Self> {
        let Ok(ipb_member_id) = env::var("EH_AUTH_ID") else {
            return None;
        };
        let Ok(ipb_pass_hash) = env::var("EH_AUTH_HASH") else {
            return None;
        };
        let igneous = env::var("EH_AUTH_IGNEOUS").ok().filter(|s| !s.is_empty());
        Some(EhClientAuth::new(
            &ipb_member_id,
            &ipb_pass_hash,
            igneous.as_deref(),
        ))
    }

    /// 输出认证 Cookie 为 `(名称, 值)` 键值对列表。
    ///
    /// 列表元素依次为 `ipb_member_id`、`ipb_pass_hash`、`igneous`（如存在），
    /// 供 [`EhClient`](crate::client::client::EhClient) 写入 CookieJar。
    #[must_use]
    pub fn to_pairs(&self) -> Vec<(String, String)> {
        let mut vec = vec![
            ("ipb_member_id".to_string(), self.ipb_member_id.clone()),
            ("ipb_pass_hash".to_string(), self.ipb_pass_hash.clone()),
        ];
        if let Some(igneous) = &self.igneous {
            vec.push(("igneous".to_string(), igneous.clone()));
        }
        vec
    }
}

impl fmt::Debug for EhClientAuth {
    /// 脱敏输出：`ipb_pass_hash` 与 `igneous` 以 `"****"` 代替，
    /// 避免凭据进入日志（`ipb_member_id` 本身是公开可见的用户编号，保留原值）。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EhClientAuth")
            .field("ipb_member_id", &self.ipb_member_id)
            .field("ipb_pass_hash", &"****")
            .field("igneous", &self.igneous.as_ref().map(|_| "****"))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::EhClientAuth;

    #[test]
    fn to_pairs_contains_all_cookies() {
        let auth = EhClientAuth::new("123456", "abcdef", Some("012345"));
        let pairs = auth.to_pairs();
        assert_eq!(
            pairs,
            vec![
                ("ipb_member_id".to_string(), "123456".to_string()),
                ("ipb_pass_hash".to_string(), "abcdef".to_string()),
                ("igneous".to_string(), "012345".to_string()),
            ]
        );
    }

    #[test]
    fn empty_igneous_is_dropped() {
        let auth = EhClientAuth::new("123456", "abcdef", Some(""));
        assert!(auth.igneous.is_none());
        assert_eq!(auth.to_pairs().len(), 2);
    }

    #[test]
    fn debug_redacts_secrets() {
        let auth = EhClientAuth::new("123456", "secret-hash", Some("secret-igneous"));
        let printed = format!("{auth:?}");
        assert!(printed.contains("123456"));
        assert!(!printed.contains("secret-hash"));
        assert!(!printed.contains("secret-igneous"));
        assert!(printed.contains("****"));
    }
}
