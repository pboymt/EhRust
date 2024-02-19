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

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let unix_timestamp = s.parse::<i64>().unwrap();
    let dt = DateTime::from_timestamp(unix_timestamp, 0).unwrap();
    Ok(dt)
}
