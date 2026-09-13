//! 内部工具（不保证 SemVer 稳定）。
//!
//! - [`regex`]：带错误处理的正则编译；
//! - [`scraper`]：CSS 选择器、文本提取、站点行内样式（评分/收藏槽位）与日期解析；
//! - [`serde`]：api.php 数字字段的字符串形式反序列化。
pub mod regex;
pub mod scraper;
pub mod serde;
