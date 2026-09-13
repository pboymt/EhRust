//! [EhTagTranslation](https://github.com/EhTagTranslation/Database) 数据库的数据结构。
//!
//! `db.text.json` 顶层结构：仓库地址、HEAD 提交信息、版本号，
//! 以及按命名空间划分的翻译数据（见 [`EttData`](crate::tags::db::EttData)）。
//! 所有字段与上游 JSON 一一对应（`frontMatters` 为 camelCase）。
//!
//! # 示例
//!
//! ```rust,no_run
//! # fn demo() -> Result<(), Box<dyn std::error::Error>> {
//! let file = std::fs::File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/db.text.json"))?;
//! let db: libeh::tags::db::EhTagTranslations = serde_json::from_reader(file)?;
//! println!("namespaces: {}", db.data.len());
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// 整份 `db.text.json`。
#[derive(Debug, Serialize, Deserialize)]
pub struct EhTagTranslations {
    /// 上游仓库地址。
    pub repo: String,
    /// 数据对应的 HEAD 提交。
    pub head: EttHead,
    /// 数据版本号。
    pub version: i32,
    /// 按命名空间划分的翻译数据。
    pub data: Vec<EttData>,
}

/// 数据对应的 git 提交信息。
#[derive(Debug, Serialize, Deserialize)]
pub struct EttHead {
    /// 提交 SHA。
    pub sha: String,
    /// 提交信息。
    pub message: String,
    /// 作者。
    pub author: EttHeadMember,
    /// 提交者。
    pub committer: EttHeadMember,
}

/// 提交成员信息。
#[derive(Debug, Serialize, Deserialize)]
pub struct EttHeadMember {
    /// 用户名。
    pub name: String,
    /// 邮箱。
    pub email: String,
    /// 时间（RFC 3339 字符串，如 `2024-01-23T05:23:05.000Z`）。
    pub when: String,
}

/// 一个命名空间的翻译集合。
///
/// `data` 的键是原始标签（如 `big breasts`），值是该标签的翻译条目。
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EttData {
    /// 命名空间（`language`/`parody`/`female`…）。
    pub namespace: String,
    /// 命名空间的元信息。
    pub front_matters: EttDataFrontMatters,
    /// 标签数量。
    pub count: isize,
    /// 标签翻译条目。
    pub data: HashMap<String, EttDataItem>,
}

/// 命名空间的元信息。
#[derive(Debug, Serialize, Deserialize)]
pub struct EttDataFrontMatters {
    /// 显示名。
    pub name: String,
    /// 描述。
    pub description: String,
    /// 命名空间键。
    pub key: String,
    /// 缩写（部分命名空间没有）。
    pub abbr: Option<String>,
    /// 别名列表（部分命名空间没有）。
    pub aliases: Option<Vec<String>>,
    /// 版权信息（可选）。
    pub copyright: Option<String>,
    /// 翻译规则。
    pub rules: Vec<String>,
    /// 示例条目（可选）。
    pub example: Option<EttDataExample>,
}

/// 单个标签的翻译条目。
#[derive(Debug, Serialize, Deserialize)]
pub struct EttDataItem {
    /// 显示名（中文翻译）。
    pub name: String,
    /// 介绍（Markdown）。
    pub intro: String,
    /// 外部链接。
    pub links: String,
}

/// 示例条目（部分上游数据带 `example` 字段）。
#[derive(Debug, Serialize, Deserialize)]
pub struct EttDataExample {
    /// 原始标签。
    pub raw: String,
    /// 显示名。
    pub name: String,
    /// 介绍。
    pub intro: String,
    /// 外部链接。
    pub links: String,
}

#[cfg(test)]
mod tests {
    use super::EhTagTranslations;

    #[test]
    fn test_parse_bundled_db() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/db.text.json");
        let file = std::fs::File::open(path).unwrap();
        let db: EhTagTranslations = serde_json::from_reader(file).unwrap();
        assert!(!db.data.is_empty());
        assert!(db.version > 0);
    }
}
