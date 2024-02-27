//! ## e-hentai/exhentai 请求与处理库
//!
//! `libeh` 提供了对 e-hentai/exhentai 的 [HTTP 客户端的封装](client)，以及[数据结构的转换器和解析器](dto)。

/// 可配置的 e-hentai/exhentai 客户端
pub mod client;
/// 数据传输对象
pub mod dto;
/// 为 [EhTagTranslation/DatabaseReleases](https://github.com/EhTagTranslation/DatabaseReleases) 设计的解析器，用于解析标签翻译
pub mod tags;
/// 对 e-hentai/exhentai 的链接构筑工具与解析工具
pub mod url;
/// 内部工具
mod utils;

/// 获取 libeh 版本号
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
