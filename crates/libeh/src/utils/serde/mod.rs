//! api.php 数字字段的字符串形式反序列化辅助。
//!
//! 站点 API 以 JSON **字符串**返回数字（`"filecount":"20"`、
//! `"rating":"4.53"`、`"posted":"1707740440"`），本模块为
//! `#[serde(with = "path")]` 提供成对的序列化/反序列化函数。
pub mod parse_float32_str;
pub mod parse_int32_str;
pub mod parse_int64_str;
pub mod parse_keyword_strings;
pub mod parse_option_int64_str;
pub mod parse_unix_timestamp_str;
