//! 搜索结果的偏移量（站点"快速分页"参数）。
//!
//! E-Hentai 的列表支持基于画廊 ID 的快速翻页（而不只是 `page=N`）：
//! 搜索 URL 携带 `prev={gid}` / `next={gid}`，可选 `jump={距离}`
//! （如 `"2"` 为两页、`"1y"` 为一年）。
//!
//! [`Offset`](crate::dto::search_offset::Offset) 实现了 `IntoIterator<Item = (String, String)>`，
//! 由 [`SearchBuilder`](crate::url::search::SearchBuilder) 写入 query 参数。
//!
//! # 示例
//!
//! ```rust
//! use libeh::dto::search_offset::Offset;
//!
//! let params: Vec<(String, String)> = Offset::Next(1234, Some("1y".into()))
//!     .into_iter()
//!     .collect();
//! assert_eq!(params, vec![("next".to_string(), "1234".to_string()), ("jump".to_string(), "1y".to_string())]);
//! ```

/// 搜索结果的偏移量。
///
/// `Prev`/`Next` 的第二个参数是可选的 `jump` 距离
/// （数字页数如 `"2"`，或 `1d`/`1w`/`1y` 等时间段）。
#[derive(Debug, Clone)]
pub enum Offset {
    /// 从指定 gid **更新**的位置继续（`prev={gid}`）。
    Prev(i64, Option<String>),
    /// 从指定 gid **更早**的位置继续（`next={gid}`）。
    Next(i64, Option<String>),
    /// 跳到结果集的某个百分比位置（0–98，新到旧；`range={percent}`）。
    Range(i64),
}

impl IntoIterator for Offset {
    type Item = (String, String);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    /// 展开为 query 参数对（`prev`/`next`/`range`，`jump` 附加在后）。
    fn into_iter(self) -> Self::IntoIter {
        let mut result = match self {
            Offset::Prev(gid, _) => vec![("prev".to_string(), gid.to_string())],
            Offset::Next(gid, _) => vec![("next".to_string(), gid.to_string())],
            Offset::Range(percent) => vec![("range".to_string(), percent.to_string())],
        };
        match self {
            Offset::Prev(_, Some(jump)) => result.push(("jump".to_string(), jump)),
            Offset::Next(_, Some(jump)) => result.push(("jump".to_string(), jump)),
            _ => {}
        };
        result.into_iter()
    }
}
