//! 标签翻译解析器，服务于
//! [EhTagTranslation/DatabaseReleases](https://github.com/EhTagTranslation/DatabaseReleases)。
//!
//! 该项目以 `db.text.json` 的形式发布 E-Hentai 标签的社区翻译
//! （中文名称、描述、外部链接等）。[`db`](crate::tags::db) 模块提供其数据结构定义，
//! 可直接用 serde 反序列化整份 JSON。
//!
//! 仓库内附带一份 `db.text.json` 样本（`crates/libeh/db.text.json`）
//! 供离线测试；生产使用建议从上游 release 拉取最新版本。

/// EhTagTranslation 数据库的数据结构。
pub mod db;
