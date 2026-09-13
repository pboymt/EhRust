//! 站点 API 数字字段的字符串形式反序列化辅助。
//!
//! api.php 的数字字段以 JSON 字符串返回（如 `"filecount":"20"`、
//! `"rating":"4.53"`），且部分数值带千分位逗号；本模块提供
//! `#[serde(with = "...")]` 用的一对序列化/反序列化函数。
//! 反序列化失败返回 serde 错误（不 panic）。

use chrono::prelude::*;
use serde::{Deserialize, Deserializer, Serializer};

#[allow(dead_code)]
pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = format!("{}", date.timestamp());
    serializer.serialize_str(&s)
}

/// 把 unix 时间戳字符串（如 `"1707740440"`）反序列化为 `DateTime<Utc>`。
pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let cleaned = s.replace(',', "");
    let unix_timestamp: i64 = cleaned.parse().map_err(serde::de::Error::custom)?;
    DateTime::from_timestamp(unix_timestamp, 0).ok_or_else(|| {
        serde::de::Error::custom(format!("unix timestamp {unix_timestamp} out of range"))
    })
}
