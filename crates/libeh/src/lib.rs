#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

//! E-Hentai / ExHentai 请求与处理库。
//!
//! `libeh` 封装了对 e-hentai.org / exhentai.org 的 HTTP 请求与页面解析：
//!
//! - [`client`]：可配置的 HTTP 客户端（认证 Cookie、代理、超时）与
//!   api.php 的高层封装；
//! - [`dto`]：数据传输对象——搜索关键词、分类、画廊信息/详情/评论/预览、
//!   搜索结果与 API 请求/响应结构，以及对应的 HTML/JSON 解析器；
//! - [`url`]：搜索页与画廊页的 URL 构建器与解析器；
//! - [`tags`]：EhTagTranslation 数据库的解析器（标签翻译）；
//! - [`error`]：统一错误类型；
//! - `utils`：内部工具（正则、scraper 辅助、serde 反序列化器）。
//!
//! # 快速上手
//!
//! ```no_run
//! use libeh::client::{client::EhClient, config::EhClientConfig};
//! use libeh::dto::keyword::Keyword;
//!
//! # async fn demo() -> Result<(), libeh::error::Error> {
//! // 读取环境变量配置（EH_SITE / EH_PROXY / EH_AUTH_*）
//! let config = EhClientConfig::env()?;
//! let client = EhClient::try_new(config)?;
//!
//! // 结构化搜索：等价于站点搜索 `artist:"simon$" language:"chinese$"`
//! let result = client
//!     .search_parsed(
//!         vec![Keyword::Artist("simon".into()), Keyword::Language("chinese".into())],
//!         None,
//!     )
//!     .await?;
//!
//! for info in &result.gallery_info_list {
//!     println!("{} ({} 页)", info.title, info.pages);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # 认证
//!
//! 站点登录态完全由 Cookie 承载（`ipb_member_id` + `ipb_pass_hash`，
//! 访问里站另需 `igneous`）。通过 [`EhClientAuth`](client::auth::EhClientAuth)
//! 或环境变量提供；客户端会把这些 Cookie 同时注入 e-hentai.org 与
//! exhentai.org 两个域，并附加 `nw=1` 跳过表站内容警告页。
//! 未提供认证时以匿名访问表站；访问里站会得到
//! [`Error::Protocol("sad panda")`](error::Error::Protocol)。
//!
//! # 离线解析
//!
//! 所有解析器（[`SearchResult`](dto::search_result::SearchResult)、
//! [`GalleryDetail`](dto::gallery::detail::GalleryDetail) 等）都是纯函数，
//! 直接接受 HTML 字符串，可脱离网络独立使用与测试。
//!
//! # 网络测试
//!
//! 集成网络测试默认 `#[ignore]`，设置环境变量 `EH_NETWORK_TESTS=1`
//! 后以 `cargo test -- --ignored` 运行（需要可用的站点连接或代理）。

/// 可配置的 e-hentai/exhentai 客户端。
pub mod client;
/// 数据传输对象与解析器。
pub mod dto;
/// 统一错误类型。
pub mod error;
/// 为 [EhTagTranslation/DatabaseReleases](https://github.com/EhTagTranslation/DatabaseReleases) 设计的解析器，用于解析标签翻译。
pub mod tags;
/// 对 e-hentai/exhentai 的链接构筑工具与解析工具。
pub mod url;
/// 内部工具。
mod utils;

/// 获取 libeh 版本号。
#[must_use]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 网络测试门控：仅在环境变量 `EH_NETWORK_TESTS=1` 时放行。
///
/// 供 `#[ignore]` 标注的网络测试在运行开始时调用；
/// 未设置门控变量时 panic 以标记"被有意跳过"，避免误报为通过。
///
/// # Panics
///
/// 环境变量 `EH_NETWORK_TESTS` 不为 `1` 时 panic。
#[doc(hidden)]
pub fn network_gate() {
    if std::env::var("EH_NETWORK_TESTS").as_deref() != Ok("1") {
        panic!("network test gated: set EH_NETWORK_TESTS=1 to run (cargo test -- --ignored)");
    }
}
