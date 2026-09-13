//! 库的统一错误类型。
//!
//! `libeh` 中所有可能失败的公开 API 统一返回 [`Result<T, Error>`](Result)。
//! 错误按产生环节分为五类，调用方可以据此决定重试、降级还是报告给用户：
//!
//! | 变体 | 产生环节 | 典型处置 |
//! |---|---|---|
//! | [`Error::Http`](crate::error::Error::Http) | 传输层失败（DNS、连接、超时、TLS） | 可指数退避后重试 |
//! | [`Error::Status`](crate::error::Error::Status) | 响应状态码非 2xx | 检查 URL 与登录态；429 应退避 |
//! | [`Error::Protocol`](crate::error::Error::Protocol) | 站点返回协议级错误（sad panda、api.php error 等） | 多数需要登录或修正参数 |
//! | [`Error::Parse`](crate::error::Error::Parse) | HTML/JSON 解析失败 | 多为站点改版；`snippet` 可用于定位 |
//! | [`Error::Config`](crate::error::Error::Config) | 本地配置非法（站点、代理、参数范围） | 修正配置后重试 |
//!
//! # 示例
//!
//! ```rust
//! use libeh::error::Error;
//!
//! fn report(err: &Error) -> String {
//!     match err {
//!         // 传输层失败：可以安全地重试
//!         Error::Http(e) => format!("network error, retry later: {e}"),
//!         // 站点明确拒绝：重试大概率无效
//!         Error::Status { code, url } => format!("got {code} from {url}"),
//!         // 常见于未登录访问 ExHentai（sad panda）
//!         Error::Protocol(msg) => format!("site says: {msg}"),
//!         // 站点改版或解析器缺陷
//!         Error::Parse { context, detail, .. } => format!("parse failed at {context}: {detail}"),
//!         // 调用方自己的配置问题
//!         Error::Config(msg) => format!("fix your config: {msg}"),
//!         // Error 标记为 #[non_exhaustive]：未来版本可能新增变体
//!         _ => "unknown error".to_string(),
//!     }
//! }
//! ```

/// 库内所有 fallible 公开 API 的统一返回类型别名。
pub type Result<T> = std::result::Result<T, Error>;
/// 库的统一错误类型。
///
/// 各变体的产生场景与处置建议见[模块文档](self)。
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// 网络请求本身失败：DNS 解析、TCP 连接、TLS 握手、请求或读取超时等。
    ///
    /// 该错误来自 [`reqwest::Error`]，通常与本地网络或代理有关，
    /// 对幂等请求（GET）可以安全地指数退避后重试。
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// 响应状态码非 2xx。
    ///
    /// 常见组合：`403`（Cloudflare 拦截）、`404`（画廊已删除）、
    /// `429`/`509`（触发站点限流或流量配额）。
    #[error("unexpected status {code} from {url}")]
    Status {
        /// 响应状态码。
        code: u16,
        /// 发出请求的目标 URL。
        url: String,
    },

    /// 站点返回的协议级错误，即 HTTP 200 但内容表示"操作无法完成"。
    ///
    /// 目前可识别（`msg` 为对应的站点原文或约定简称）：
    /// - `"sad panda"`：未登录或 `igneous` 失效时访问 ExHentai；
    /// - `"kokomade"`：同上，站点返回日文引导页（`exhentai.org/img/kokomade.jpg`）；
    /// - `"gallery unavailable"`：画廊不可见或已删除；
    /// - `"offensive"` / `"pining"`：画廊被标记为冒犯性内容或已进入删除流程；
    /// - api.php 响应中 `error` 字段的原文；
    /// - `"This page requires you to log on."`：收藏夹等页面要求登录。
    #[error("site error: {0}")]
    Protocol(String),

    /// HTML 或 JSON 解析失败。
    ///
    /// 绝大多数情况下意味着站点页面结构发生改版（解析器滞后），
    /// `snippet` 保留了失败位置的原始内容片段，便于离线排查；
    /// `context` 是失败发生的解析环节（如 `"search nav"`、`"gallery detail"`）。
    #[error("failed to parse {context}: {detail}")]
    Parse {
        /// 失败发生的解析环节，如 `"search nav"`、`"gallery detail"`。
        context: &'static str,
        /// 失败的具体原因。
        detail: String,
        /// 原始响应中与失败相关的片段（截断后），用于离线排查。
        snippet: String,
    },

    /// 本地配置非法：站点无法识别、代理 URL 无法解析、请求参数超出站点允许范围等。
    #[error("invalid configuration: {0}")]
    Config(String),
}

impl Error {
    /// 以解析环节、原因与原始片段构造 [`Error::Parse`]。
    pub fn parse(
        context: &'static str,
        detail: impl Into<String>,
        snippet: impl Into<String>,
    ) -> Self {
        Error::Parse {
            context,
            detail: detail.into(),
            snippet: snippet.into(),
        }
    }

    /// 构造 [`Error::Parse`]，`detail` 为固定描述（不携带动态原因）。
    pub fn parse_msg(
        context: &'static str,
        detail: &'static str,
        snippet: impl Into<String>,
    ) -> Self {
        Error::Parse {
            context,
            detail: detail.to_string(),
            snippet: snippet.into(),
        }
    }

    /// 构造 [`Error::Status`]。
    pub fn status(code: u16, url: impl Into<String>) -> Self {
        Error::Status {
            code,
            url: url.into(),
        }
    }
}

/// 从原始响应文本中截取一段用于 [`Error::Parse::snippet`] 的片段。
///
/// 超出 `max` 字节时按字符边界截断并追加 `…[truncated]` 标记，
/// 避免错误信息携带整个页面。
pub(crate) fn snippet(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_string();
    }
    let mut end = max;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…[truncated]", &text[..end])
}
