//! 站点 API 数字字段的字符串形式反序列化辅助。
//!
//! api.php 的数字字段以 JSON 字符串返回（如 `"filecount":"20"`、
//! `"rating":"4.53"`），且部分数值带千分位逗号；本模块提供
//! `#[serde(with = "...")]` 用的一对序列化/反序列化函数。
//! 反序列化失败返回 serde 错误（不 panic）。

use serde::{Deserialize, Deserializer, Serializer};

#[allow(dead_code)]
pub fn serialize<S>(number: &Option<i64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match number {
        Some(n) => serializer.serialize_str(&n.to_string()),
        None => serializer.serialize_none(),
    }
}

/// 把可缺省的数字字符串反序列化为 `Option<i64>`：
/// 字段缺失、空串或无法解析时为 `None`（站点用空串表达"无父画廊"）。
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let cleaned = s.replace(',', "");
    Ok(cleaned.parse::<i64>().ok())
}
