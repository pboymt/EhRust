use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serializer};

use crate::dto::keyword::Keyword;

#[allow(dead_code)]
pub fn serialize<S>(keywords: Vec<Keyword>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(keywords.len()))?;
    for keyword in &keywords {
        seq.serialize_element(&keyword.to_string())?;
    }
    seq.end()
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Keyword>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Vec<String> = Vec::deserialize(deserializer)?;
    let mut keywords: Vec<Keyword> = vec![];
    for s in s {
        let keyword = Keyword::from(s);
        keywords.push(keyword);
    }
    Ok(keywords)
}
