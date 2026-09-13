//! api.php 的请求与响应数据封装。
//!
//! E-Hentai 的 JSON API 官方文档见 <https://ehwiki.org/wiki/API>。
//! 本模块目前覆盖两个方法：
//!
//! - `gdata`：由画廊 ID + 令牌批量检索元数据（[`GalleryMetadataRequest`](crate::dto::api::GalleryMetadataRequest)）；
//! - `gtoken`：由画廊 ID + 页面令牌 + 页号反查画廊令牌（[`GalleryTokensRequest`](crate::dto::api::GalleryTokensRequest)）。
//!
//! # 协议要点
//!
//! - `gdata` 的 `gidlist` 单次请求**最多 25 条**（[`GDATA_MAX_ITEMS`](crate::dto::api::GDATA_MAX_ITEMS)），
//!   超出会被站点拒绝；[`EhClient::gallery_metadata`](crate::client::client::EhClient::gallery_metadata)
//!   已自动分批；
//! - `namespace: 1` 使响应中的 `tags` 携带命名空间前缀（`language:chinese`）；
//! - 所有数字字段以**字符串**形式返回（如 `"rating":"4.53"`），
//!   本模块通过 `crate::utils::serde` 的自定义反序列化器处理；
//! - 对无效的 gid/token，响应的 `gmetadata` 条目会是 `{"error": ...}` 对象，
//!   以 [`GalleryMetadataError`](crate::dto::api::GalleryMetadataError) 表示。
//!
//! # 示例
//!
//! 构造一个 `gdata` 请求体：
//!
//! ```rust
//! use libeh::dto::api::{GidListItem, GalleryMetadataRequest};
//!
//! let item = GidListItem::try_from("https://e-hentai.org/g/2791585/3e7e1c7107/".to_string())?;
//! let request = GalleryMetadataRequest::new(vec![item])?;
//! let json = serde_json::to_string(&request).unwrap();
//! assert!(json.contains(r#""method":"gdata""#));
//! # Ok::<(), libeh::error::Error>(())
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::utils::regex::regex;
use crate::utils::serde::{
    parse_float32_str, parse_int32_str, parse_int64_str, parse_keyword_strings,
    parse_option_int64_str, parse_unix_timestamp_str,
};

use super::keyword::Keyword;

use std::str::FromStr;
use std::sync::LazyLock;

/// api.php 表站地址。
pub const API_URL_EH: &str = "https://api.e-hentai.org/api.php";
/// api.php 里站地址（需要认证 Cookie）。
pub const API_URL_EX: &str = "https://api.exhentai.org/api.php";
/// `gdata` 方法单次请求允许的最大画廊条目数（站点上限）。
pub const GDATA_MAX_ITEMS: usize = 25;

/// 画廊 URL 的解析规则：`/g/{gid}/{token}/`，
/// 同时接受 `mpv` 路径与尾部斜杠/锚点。
static GALLERY_URL_PATTERN: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex(r"^https://e[\-x]hentai\.org/(?:g|mpv)/(?<gid>\d+)/(?<token>[0-9a-f]{10})/?")
        .expect("constant regex")
});

/// 页面 URL 的解析规则：`/s/{ptoken}/{gid}-{page}`。
static PAGE_URL_PATTERN: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex(r"^https://e[\-x]hentai\.org/s/(?<ptoken>[0-9a-f]+)/(?<gid>\d+)-(?<pnum>\d+)/?")
        .expect("constant regex")
});

/// 画廊 ID 及其令牌（`gdata` 的 `gidlist` 条目）。
///
/// 序列化形式为 `[gid, "token"]` 二元组。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GidListItem(pub i64, pub String);

impl GidListItem {
    /// 新建画廊 ID 及其令牌。
    #[must_use]
    pub fn new(gid: i64, token: &str) -> Self {
        GidListItem(gid, token.into())
    }
}

impl TryFrom<String> for GidListItem {
    type Error = Error;

    /// 从画廊 URL 字符串解析（接受 `mpv` 链接与尾部斜杠/锚点）。
    ///
    /// # Errors
    ///
    /// URL 不符合 `https://(e-hentai|exhentai).org/(g|mpv)/{gid}/{token}/` 形态时
    /// 返回 [`Error::Parse`]。
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let caps = GALLERY_URL_PATTERN
            .captures(&value)
            .ok_or_else(|| Error::parse("gallery url", "not a gallery url", value.clone()))?;
        let gid = caps["gid"]
            .parse::<i64>()
            .map_err(|e| Error::parse("gallery url gid", e.to_string(), value.clone()))?;
        Ok(GidListItem(gid, caps["token"].to_string()))
    }
}

impl FromStr for GidListItem {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        GidListItem::try_from(s.to_string())
    }
}

impl From<GidListItem> for String {
    fn from(item: GidListItem) -> Self {
        format!("https://e-hentai.org/g/{}/{}/", item.0, item.1)
    }
}

/// 历史别名：旧版命名 [`GidListItem`] 为 `GIDListItem`。
#[deprecated(since = "0.2.0", note = "renamed to `GidListItem`")]
pub type GIDListItem = GidListItem;

/// 通过画廊 ID 及其令牌检索元数据的请求体（`gdata`）。
///
/// `gidlist` 单次最多 [`GDATA_MAX_ITEMS`] 条，超出站点会拒绝整个请求。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryMetadataRequest {
    /// 请求方法，恒为 `"gdata"`。
    pub method: String,
    /// 画廊 ID 及其令牌列表。
    pub gidlist: Vec<GidListItem>,
    /// 元数据命名空间，恒为 `1`，否则 `tags` 不含命名空间前缀。
    pub namespace: i32,
}

impl GalleryMetadataRequest {
    /// 构造 `gdata` 请求体。
    ///
    /// # Errors
    ///
    /// `gidlist` 超过 [`GDATA_MAX_ITEMS`]（25）条时返回 [`Error::Config`]；
    /// 需要大批量查询时请使用
    /// [`EhClient::gallery_metadata`](crate::client::client::EhClient::gallery_metadata)
    /// 的自动分批。
    pub fn new(gidlist: Vec<GidListItem>) -> Result<Self, Error> {
        if gidlist.len() > GDATA_MAX_ITEMS {
            return Err(Error::Config(format!(
                "gidlist has {} items; the site accepts at most {GDATA_MAX_ITEMS} per gdata request",
                gidlist.len()
            )));
        }
        Ok(Self {
            method: "gdata".into(),
            gidlist,
            namespace: 1,
        })
    }
}

/// 画廊种子数据（`gdata` 响应中的 `torrents` 数组条目）。
#[derive(Debug, Clone, Deserialize)]
pub struct GalleryTorrent {
    /// 种子 infohash。
    pub hash: String,
    /// 添加时间。
    #[serde(with = "parse_unix_timestamp_str")]
    pub added: DateTime<Utc>,
    /// 种子名称。
    pub name: String,
    /// 种子文件总大小（字节）。
    #[serde(with = "parse_int64_str")]
    pub tsize: i64,
    /// 画廊文件总大小（字节）。
    #[serde(with = "parse_int64_str")]
    pub fsize: i64,
}

/// 画廊元数据（`gdata` 响应条目）。
///
/// 数字字段以字符串形式返回，由 `crate::utils::serde` 反序列化。
#[derive(Debug, Clone, Deserialize)]
pub struct GalleryMetadata {
    /// 画廊 ID。
    pub gid: i64,
    /// 画廊版本令牌。
    pub token: String,
    /// 归档下载的 key（`archiver.php` 用），可能缺失。
    pub archiver_key: Option<String>,
    /// 标题（罗马字/英文）。
    pub title: String,
    /// 标题（日文），可能为空串。
    pub title_jpn: String,
    /// 分类文本（如 `"Artist CG"`），可用 [`crate::dto::gallery::category::Category`] 解析。
    pub category: String,
    /// 缩略图 URL。
    pub thumb: String,
    /// 上传者；画廊被 disown 后为空串。
    pub uploader: String,
    /// 上传时间（真 UTC，来自 unix 时间戳；与 HTML 页面渲染时间可能因账号时区不同）。
    #[serde(with = "parse_unix_timestamp_str")]
    pub posted: DateTime<Utc>,
    /// 页数。
    #[serde(with = "parse_int32_str")]
    pub filecount: i32,
    /// 文件总大小（字节）。
    pub filesize: i64,
    /// 是否已被删除（expunged）。
    pub expunged: bool,
    /// 评分（0.0–5.0）。
    #[serde(with = "parse_float32_str")]
    pub rating: f32,
    /// 种子数量。
    #[serde(with = "parse_int32_str")]
    pub torrentcount: i32,
    /// 种子列表。
    pub torrents: Vec<GalleryTorrent>,
    /// 标签（`namespace:1` 时带命名空间前缀）。
    #[serde(with = "parse_keyword_strings")]
    pub tags: Vec<Keyword>,
    /// 父画廊 ID（无父子关系时缺失）。
    #[serde(with = "parse_option_int64_str")]
    pub parent_gid: Option<i64>,
    /// 父画廊令牌。
    pub parent_key: Option<String>,
    /// 系列中最早的画廊 ID。
    #[serde(with = "parse_option_int64_str")]
    pub first_gid: Option<i64>,
    /// 系列中最早的画廊令牌。
    pub first_key: Option<String>,
}

/// `gdata` 成功响应体。
#[derive(Debug, Clone, Deserialize)]
pub struct GalleryMetadataResponse {
    /// 元数据条目数组。
    pub gmetadata: Vec<GalleryMetadata>,
}

/// `gdata` 响应中的单条目错误（无效的 gid/token）。
#[derive(Debug, Clone, Deserialize)]
pub struct GalleryMetadataError {
    /// 出错的画廊 ID（响应未提供时为 `-1`）。
    pub gid: i64,
    /// 错误描述。
    pub error: String,
}

/// 页面列表条目（`gtoken` 的 `pagelist`）：画廊 ID、页面令牌和页号。
///
/// 序列化形式为 `[gid, "ptoken", page]` 三元组。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageListItem(pub i64, pub String, pub i32);

impl PageListItem {
    /// 新建页面列表条目。
    #[must_use]
    pub fn new(gid: i64, ptoken: &str, page: i32) -> Self {
        PageListItem(gid, ptoken.into(), page)
    }
}

impl TryFrom<String> for PageListItem {
    type Error = Error;

    /// 从页面 URL 字符串解析（接受尾部斜杠/锚点）。
    ///
    /// # Errors
    ///
    /// URL 不符合 `https://(e-hentai|exhentai).org/s/{ptoken}/{gid}-{page}/` 形态时
    /// 返回 [`Error::Parse`]。
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let caps = PAGE_URL_PATTERN
            .captures(&value)
            .ok_or_else(|| Error::parse("page url", "not a page url", value.clone()))?;
        let gid = caps["gid"]
            .parse::<i64>()
            .map_err(|e| Error::parse("page url gid", e.to_string(), value.clone()))?;
        let pnum = caps["pnum"]
            .parse::<i32>()
            .map_err(|e| Error::parse("page url number", e.to_string(), value.clone()))?;
        Ok(PageListItem(gid, caps["ptoken"].to_string(), pnum))
    }
}

impl FromStr for PageListItem {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PageListItem::try_from(s.to_string())
    }
}

/// 由画廊 ID、页面令牌和页号反查画廊令牌的请求体（`gtoken`）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryTokensRequest {
    /// 请求方法，恒为 `"gtoken"`。
    pub method: String,
    /// 页面列表。
    pub pagelist: Vec<PageListItem>,
}

impl GalleryTokensRequest {
    /// 构造 `gtoken` 请求体。
    #[must_use]
    pub fn new(pagelist: Vec<PageListItem>) -> Self {
        Self {
            method: "gtoken".to_string(),
            pagelist,
        }
    }
}

/// 通过 api.php `showpage` 方法取图片页数据的请求体。
///
/// 与 HTML 页面（`/s/…`）相比，`showpage` 一次返回图片地址与操作区数据，
/// 且不消耗页面浏览计数；`imgkey` 即页面令牌 `pToken`。
///
/// # 示例
///
/// ```rust
/// use libeh::dto::api::GalleryPageApiRequest;
///
/// let request = GalleryPageApiRequest::new(618395, 2, "0439fa3666", "530350-8".into());
/// assert_eq!(request.method, "showpage");
/// assert_eq!(request.page, 3); // 入参为 0 基页号，请求体使用 1 基页号
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryPageApiRequest {
    /// 请求方法，恒为 `"showpage"`。
    pub method: String,
    /// 画廊 ID。
    pub gid: i64,
    /// 页号（**1 基**；站点协议如此）。
    pub page: i32,
    /// 页面令牌（`pToken`）。
    pub imgkey: String,
    /// 翻页会话参数，来自 [`crate::dto::gallery::page::GalleryPage::show_key`]。
    pub showkey: String,
}

impl GalleryPageApiRequest {
    /// 构造 `showpage` 请求体。
    ///
    /// `page` 为 **0 基**页号（与 [`PageListItem`] 的页号语义一致），
    /// 内部转换为协议要求的 1 基。
    #[must_use]
    pub fn new(gid: i64, page: i32, imgkey: &str, showkey: String) -> Self {
        Self {
            method: "showpage".into(),
            gid,
            page: page + 1,
            imgkey: imgkey.into(),
            showkey,
        }
    }
}

/// api.php `showpage` 的响应解析结果。
///
/// 响应为 `{"i3":"…","i5":"…","i6":"…","i7":…}` 形态，
/// 每个字段是一段内嵌 HTML，从中提取地址与操作参数。
/// 语义与 HTML 页面解析器 [`crate::dto::gallery::page::GalleryPage`] 一致。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalleryPageApiResult {
    /// 当前页图片地址。
    pub image_url: String,
    /// 换源参数（`nl('…')`；页面未提供时为 `None`）。
    pub skip_hath_key: Option<String>,
    /// 原图页链接（`fullimg.php?…`，需要原片权限）。
    pub origin_image_url: Option<String>,
    /// 原图直链（`prompt('Copy the URL below.', '…')` 给出）。
    pub other_image_url: Option<String>,
}

/// 解析 api.php `showpage` 的原始响应文本。
///
/// # Errors
///
/// 响应含顶层 `error` 字段 → [`Error::Protocol`]；
/// 缺少 `i3` 或其中没有图片地址 → [`Error::Parse`]。
pub fn parse_gallery_page_response(body: &str) -> Result<GalleryPageApiResult, Error> {
    use crate::error::snippet;

    static IMAGE_URL: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r#"<img[^>]*src="([^"]+)"[^>]*style"#).expect("constant regex")
    });
    static SKIP_HATH: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"return nl\('([^)]+)'\)").expect("constant regex"));
    static ORIGIN_PROMPT: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"prompt\('Copy the URL below\.', '([^']+)'\)").expect("constant regex")
    });
    static ORIGIN_FULLIMG: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r#"<a href="([^"]*fullimg[^"]*)">"#).expect("constant regex")
    });

    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|e| Error::parse("showpage response", e.to_string(), snippet(body, 512)))?;
    if let Some(err) = value.get("error").and_then(serde_json::Value::as_str) {
        return Err(Error::Protocol(err.to_string()));
    }
    let i3 = value
        .get("i3")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| Error::parse("showpage response", "missing i3", snippet(body, 512)))?;
    let image_url = IMAGE_URL
        .captures(i3)
        .map(|c| crate::utils::unescape_xml(c[1].trim()))
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::parse("showpage response", "no image in i3", snippet(i3, 512)))?;

    let i6 = value
        .get("i6")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let i7 = value
        .get("i7")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    // 原图链接优先取 i7 的 fullimg.php（旧式协议），缺失时回退 i6 的 prompt 直链
    let origin_from_i7 = ORIGIN_FULLIMG
        .captures(i7)
        .map(|c| crate::utils::unescape_xml(c[1].trim()));
    let other = ORIGIN_PROMPT
        .captures(i6)
        .map(|c| crate::utils::unescape_xml(c[1].trim()));

    Ok(GalleryPageApiResult {
        image_url,
        skip_hath_key: SKIP_HATH
            .captures(i6)
            .map(|c| crate::utils::unescape_xml(c[1].trim())),
        origin_image_url: origin_from_i7,
        other_image_url: other,
    })
}

/// 画廊 ID 与令牌（`gtoken` 响应条目）。
#[derive(Debug, Clone, Deserialize)]
pub struct TokenListItem {
    /// 画廊 ID。
    pub gid: i64,
    /// 画廊令牌。
    pub token: String,
}

/// `gtoken` 成功响应体。
#[derive(Debug, Clone, Deserialize)]
pub struct GalleryTokenResponse {
    /// 令牌条目数组。
    pub tokenlist: Vec<TokenListItem>,
}

#[cfg(test)]
mod tests {
    use super::{GalleryMetadataRequest, GidListItem, PageListItem};

    #[test]
    fn test_gid_list_item() {
        let item = GidListItem::try_from("https://e-hentai.org/g/2231376/a7584a5932/".to_string())
            .unwrap();
        assert_eq!(item.0, 2231376);
        assert_eq!(item.1, "a7584a5932");
    }

    #[test]
    fn gid_item_accepts_mpv_and_ex() {
        let a =
            GidListItem::try_from("https://exhentai.org/mpv/2519745/76939e430f/#page1".to_string());
        assert!(a.is_ok());
        let b = GidListItem::try_from("https://e-hentai.org/g/123/not-a-token".to_string());
        assert!(b.is_err());
    }

    #[test]
    fn test_page_list_item() {
        let item =
            PageListItem::try_from("https://e-hentai.org/s/40bc07a79a/618395-11".to_string())
                .unwrap();
        assert_eq!(item.0, 618395);
        assert_eq!(item.1, "40bc07a79a");
        assert_eq!(item.2, 11);
    }

    #[test]
    fn invalid_urls_are_errors_not_panics() {
        assert!(GidListItem::try_from("not a url".to_string()).is_err());
        assert!(
            PageListItem::try_from("https://e-hentai.org/g/1/abcd123456/".to_string()).is_err()
        );
    }

    #[test]
    fn test_parse_gallery_page_response() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/pending/GalleryPageApiParserTest.json"
        );
        let body = std::fs::read_to_string(path).unwrap();
        let result = super::parse_gallery_page_response(&body).unwrap();
        assert!(result.image_url.contains("/h/"), "{}", result.image_url);
        // i7 的 fullimg 链接（HTML 实体已反转义）
        assert!(
            result
                .origin_image_url
                .as_deref()
                .is_some_and(|u| u.contains("fullimg.php?gid=1366222") && !u.contains("&amp;")),
            "{:?}",
            result.origin_image_url
        );
        // i6 的 prompt 直链
        assert!(
            result
                .other_image_url
                .as_deref()
                .is_some_and(|u| u.contains("/r/")),
            "{:?}",
            result.other_image_url
        );
    }

    #[test]
    fn test_gallery_page_response_error_field() {
        let err = super::parse_gallery_page_response(r#"{"error":"Invalid page."}"#).unwrap_err();
        assert!(matches!(err, crate::error::Error::Protocol(m) if m == "Invalid page."));
    }

    #[tokio::test]
    #[ignore = "需要网络与代理；设置 EH_NETWORK_TESTS=1 并配置 .env 后运行 `cargo test -- --ignored`"]
    async fn test_gallery_metadata_request() {
        use crate::client::{client::EhClient, config::EhClientConfig};
        crate::network_gate();
        let config = EhClientConfig::env().unwrap();
        let client = EhClient::try_new(config).unwrap();
        let body = GalleryMetadataRequest::new(vec![GidListItem::try_from(
            "https://e-hentai.org/g/2791585/3e7e1c7107/".to_string(),
        )
        .unwrap()]);
        let results = client
            .gallery_metadata(vec![GidListItem::new(2791585, "3e7e1c7107")])
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        let _ = body;
    }

    #[tokio::test]
    #[ignore = "需要网络与代理；设置 EH_NETWORK_TESTS=1 并配置 .env 后运行 `cargo test -- --ignored`"]
    async fn test_gallery_token_request() {
        use crate::client::{client::EhClient, config::EhClientConfig};
        crate::network_gate();
        let config = EhClientConfig::env().unwrap();
        let client = EhClient::try_new(config).unwrap();
        let results = client
            .gallery_tokens(vec![PageListItem::try_from(
                "https://e-hentai.org/s/d384d63ec0/2519745-8".to_string(),
            )
            .unwrap()])
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].gid, 2519745);
        assert_eq!(results[0].token, "76939e430f");
    }
}
