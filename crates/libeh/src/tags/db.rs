use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EhTagTranslations {
    pub repo: String,
    pub head: ETTHead,
    pub version: i32,
    pub data: Vec<ETTData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ETTHead {
    pub sha: String,
    pub message: String,
    pub author: ETTHeadMember,
    pub committer: ETTHeadMember,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ETTHeadMember {
    name: String,
    email: String,
    when: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ETTData {
    namespace: String,
    front_matters: ETTDataFrontMatters,
    count: isize,
    data: HashMap<String, ETTDataItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ETTDataFrontMatters {
    name: String,
    description: String,
    key: String,
    abbr: Option<String>,
    aliases: Option<Vec<String>>,
    copyright: Option<String>,
    rules: Vec<String>,
    example: Option<ETTDataExample>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ETTDataItem {
    name: String,
    intro: String,
    links: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ETTDataExample {
    raw: String,
    name: String,
    intro: String,
    links: String,
}

#[test]
fn test_eh_tag_translations() {
    use std::fs::File;
    let file = File::open("db.text.json").unwrap();
    let eh_tag_translations: EhTagTranslations = serde_json::from_reader(file).unwrap();
    println!("eh_tag_translations: {:#?}", eh_tag_translations.data.len());
}
