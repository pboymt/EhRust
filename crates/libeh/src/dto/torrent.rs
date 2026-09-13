//! 画廊种子列表页（`gallerytorrents.php`）的解析器。
//!
//! 种子弹窗页每个种子是一个 `<form action="…gallerytorrents.php?…">` 块，
//! 块内为键值表（`<td>` 的首个 `<span>` 是加粗标签，如 `Posted:`/`Size:`，
//! 其余文本为值）与 `https://ehtracker.org/get/{id}/{hash}.torrent` 下载锚点。
//!
//! # 分享礼仪
//!
//! 下载锚点可能携带 `?p={个人私钥}` 查询参数（部分页面版本才有）；
//! 该参数会把下载者的身份编进链接，**公开分享前必须剥离**——
//! [`TorrentEntry::url`](crate::dto::torrent::TorrentEntry::url) 已自动完成（EhViewer 同款礼仪）。
//!
//! # 示例
//!
//! ```rust,no_run
//! # use libeh::dto::torrent::TorrentEntry;
//! # fn demo(html: String) -> Result<(), libeh::error::Error> {
//! for torrent in TorrentEntry::parse_page(&html)? {
//!     println!("{} ({} seeds) -> {}", torrent.name, torrent.seeds.unwrap_or(0), torrent.url);
//! }
//! # Ok(())
//! # }
//! ```

use chrono::{DateTime, NaiveDateTime, Utc};
use scraper::Html;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::utils::scraper::{selector, text_content};

/// 画廊的一个种子条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentEntry {
    /// 种子下载地址（`ehtracker.org/get/…`，已剥离 `?p=` 私钥参数）。
    pub url: String,
    /// 种子名称（通常与画廊标题一致）。
    pub name: String,
    /// 发布时间（页面渲染文本按 UTC 解释）。
    pub posted: Option<DateTime<Utc>>,
    /// 体积（页面渲染文本，如 `"252.0 MiB"`）。
    pub size: String,
    /// 上传者。
    pub uploader: Option<String>,
    /// 做种数。
    pub seeds: Option<i64>,
    /// 下载次数。
    pub downloads: Option<i64>,
}

impl TorrentEntry {
    /// 解析种子列表页 HTML（每个 `<form>` 块一个种子）。
    ///
    /// # Errors
    ///
    /// 仅在 CSS 选择器非法时失败（正常页面即便没有种子也返回空列表）。
    pub fn parse_page(html: &str) -> Result<Vec<Self>, Error> {
        let d = Html::parse_document(html);
        let form_sel = selector("form[action*='gallerytorrents.php']")?;
        let a_sel = selector("a[href*='.torrent']")?;
        let td_sel = selector("td")?;
        let span_sel = selector("span")?;
        let mut entries = Vec::new();

        for form in d.select(&form_sel) {
            let Some(anchor) = form.select(&a_sel).next() else {
                continue;
            };
            let Some(raw_url) = anchor.value().attr("href") else {
                continue;
            };
            // 剥离 ?p= 个人私钥参数，使种子可公开分享
            let url = match raw_url.find("?p=") {
                Some(idx) => raw_url[..idx].to_string(),
                None => raw_url.to_string(),
            };
            let name = text_content(anchor.text());
            if name.is_empty() {
                continue;
            }

            let mut entry = TorrentEntry {
                url,
                name,
                posted: None,
                size: String::new(),
                uploader: None,
                seeds: None,
                downloads: None,
            };

            // 键值提取：<td> 的首个 <span> 是加粗标签（"Posted:" 等），
            // 去掉标签后的剩余文本即值
            for td in form.select(&td_sel) {
                let Some(label_span) = td.select(&span_sel).next() else {
                    continue;
                };
                let label = text_content(label_span.text());
                let full = text_content(td.text());
                let value = full
                    .strip_prefix(&label)
                    .unwrap_or(&full)
                    .trim()
                    .to_string();
                match label.as_str() {
                    "Posted:" => {
                        entry.posted = NaiveDateTime::parse_from_str(&value, "%Y-%m-%d %H:%M")
                            .ok()
                            .map(|naive| naive.and_utc());
                    }
                    "Size:" => entry.size = value,
                    "Uploader:" => entry.uploader = Some(value).filter(|s| !s.is_empty()),
                    "Seeds:" => entry.seeds = value.parse().ok(),
                    "Downloads:" => entry.downloads = value.parse().ok(),
                    _ => {}
                }
            }
            entries.push(entry);
        }
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::TorrentEntry;

    #[test]
    fn parses_torrent_list_fixture() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/pending/torrentList.html"
        );
        let html = std::fs::read_to_string(path).unwrap();
        let torrents = TorrentEntry::parse_page(&html).unwrap();
        assert_eq!(torrents.len(), 2, "two torrents in fixture");
        let first = &torrents[0];
        assert!(first.url.starts_with("https://ehtracker.org/get/"));
        assert!(first.url.ends_with(".torrent"));
        assert!(!first.url.contains("?p="), "private key must be stripped");
        assert!(first.name.contains("Uniyaa"));
        assert_eq!(
            first.posted.map(|d| d.to_string()),
            Some("2026-04-26 05:14:00 UTC".to_string())
        );
        assert_eq!(first.size, "252.0 MiB");
        assert_eq!(first.uploader.as_deref(), Some("Konazumi"));
        assert_eq!(first.seeds, Some(13));
        assert_eq!(first.downloads, Some(50));
    }
}
