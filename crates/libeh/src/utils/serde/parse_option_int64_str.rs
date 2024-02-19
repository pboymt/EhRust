use serde::{Deserialize, Deserializer, Serializer};

#[allow(dead_code)]
pub fn serialize<S>(number: Option<i64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match number {
        Some(n) => {
            let s = n.to_string();
            serializer.serialize_str(&s)
        }
        None => serializer.serialize_none(),
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer);
    match s {
        Ok(s) => match s.parse::<i64>() {
            Ok(n) => Ok(Some(n)),
            Err(_) => Ok(None),
        },
        Err(_) => Ok(None),
    }
}
