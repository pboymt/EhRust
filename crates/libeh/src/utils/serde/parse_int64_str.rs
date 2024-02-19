use serde::{Deserialize, Deserializer, Serializer};

#[allow(dead_code)]
pub fn serialize<S>(number: i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = number.to_string();
    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let number = s.parse::<i64>().unwrap();
    Ok(number)
}
