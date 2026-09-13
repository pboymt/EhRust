//! 站点 API 数字字段的字符串形式反序列化辅助。
//!
//! api.php 的数字字段以 JSON 字符串返回（如 `"filecount":"20"`、
//! `"rating":"4.53"`），且部分数值带千分位逗号；本模块提供
//! `#[serde(with = "...")]` 用的一对序列化/反序列化函数。
//! 反序列化失败返回 serde 错误（不 panic）。

use serde::{Deserialize, Deserializer, Serializer};

#[allow(dead_code)]
pub fn serialize<S>(number: i32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = number.to_string();
    serializer.serialize_str(&s)
}

/// 把形如 `"1,234"` / `"4.53"` 的字符串反序列化为 `i32`。
pub fn deserialize<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let cleaned = s.replace(',', "");
    cleaned.parse::<i32>().map_err(serde::de::Error::custom)
}
