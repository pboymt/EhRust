//! URL 构建器与解析器。
//!
//! 站点 URL 模板一览：
//!
//! | 用途 | 模板 |
//! |---|---|
//! | 画廊详情 | `/g/{gid}/{token}/`（可选 `?p={预览页}`） |
//! | 多页阅读 | `/mpv/{gid}/{token}/` |
//! | 图片页 | `/s/{pToken}/{gid}-{页号}` |
//! | 搜索/列表 | `/`（query 参数见 [`SearchBuilder`](crate::url::search::SearchBuilder)） |
//! | 订阅 | `/watched` |
//! | 上传者/标签搜索 | `/uploader/{名称}`、`/tag/{namespace}:{tag}` |
//!
//! - [`search`](crate::url::search)：搜索 URL 构建（分类掩码、关键词、高级搜索、偏移量）；
//! - [`gallery`](crate::url::gallery)：画廊 URL 构建（[`GalleryBuilder`](crate::url::gallery::GalleryBuilder)）与
//!   从任意文本解析画廊链接。
pub mod gallery;
pub mod search;
