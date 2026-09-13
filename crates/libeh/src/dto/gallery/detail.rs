//! 画廊详情页的解析器。
//!
//! 输入为 `/g/{gid}/{token}/` 详情页的 HTML，输出 [`GalleryDetail`](crate::dto::gallery::detail::GalleryDetail)。
//! 与 EhViewer 的 `GalleryDetailParser` 对齐：
//!
//! - **会话凭据**：从内联脚本提取 `var gid/token/apiuid/apikey`——
//!   `apiuid`/`apikey` 是评分、评论投票等 api.php 调用的会话参数；
//! - **特殊页嗅探**：Offensive（冒犯性警告）与 Pining（删除中）页面
//!   返回 [`Error::Protocol`](crate::error::Error::Protocol)，`class="d"` 错误框文案同样如此；
//! - **元数据**：标题（`#gn`/`#gj`）、分类（`#gdc`）、上传者（`#gdn`）、
//!   键值表（`#gdd`：Posted/Parent/Visible/Language/File Size/Length/Favorited）、
//!   评分（`#rating_count`/`#rating_label`）、收藏态（`#gdf`）、
//!   新版本（`#gnd`）、标签（`#taglist`）；
//! - **功能链接**：Torrent/Archive 弹窗链接按**文本锚定**提取
//!   （`Torrent Download (N)` / `Archive Download`），
//!   不依赖 `#gd5` 内条目的固定顺序；
//! - **预览**：调用 [`GalleryPreview::parse`](crate::dto::gallery::preview::GalleryPreview::parse) 填充 `preview` 字段
//!   （新旧版式自动识别）。
//!
//! # 示例
//!
//! ```rust,no_run
//! # use libeh::dto::gallery::detail::GalleryDetail;
//! # fn demo(html: String) -> Result<(), libeh::error::Error> {
//! let detail = GalleryDetail::parse(html)?;
//! println!("{}: api_key={}", detail.info.title, detail.api_key);
//! for comment in &detail.comments {
//!     println!("{}: {}", comment.user, comment.score);
//! }
//! # Ok(())
//! # }
//! ```

use std::str::FromStr;

use chrono::{DateTime, Utc};
use scraper::Element as _;
use scraper::Html;
use serde::{Deserialize, Serialize};

use crate::dto::gallery::{category::Category, comment::GalleryComment, preview::GalleryPreview};
use crate::error::{snippet, Error};
use crate::url::gallery::GalleryBuilder;
use crate::utils::scraper::{parse_posted, parse_to, selector, text_content};

use super::info::GalleryInfo;

/// 内联脚本中的画廊核心参数（gid/token/apiuid/apikey相邻出现）。
const PATTERN_DETAIL: &str = r#"var gid = (?<gid>\d+);[\s\S]*?var token = "(?<token>[a-f0-9]+)";[\s\S]*?var apiuid = (?<apiuid>-?\d+);[\s\S]*?var apikey = "(?<apikey>[a-f0-9]+)";"#;
/// Torrent 弹窗链接（`popUp('…')` onclick）。
const PATTERN_TORRENT_ONCLICK: &str = r#"return popUp\('(?<link>[^']+)'[^)]+\)"#;
/// Torrent 数量（锚点文本 `Torrent Download (N)`）。
const PATTERN_TORRENT_COUNT: &str = r#"Torrent Download \((?<count>\d+)\)"#;
/// 封面行内样式中的图片 URL。
const PATTERN_COVER: &str =
    r#"width:(?<width>\d+)px; height:(?<height>\d+)px.+?url\((?<link>.+?)\)"#;
/// 页数文本（`#gdd` 的 Length 行）。
const PATTERN_PAGES_TEXT: &str = r"(?<length>[\d,]+) pages";
/// 收藏次数文本（`Favorited N times`）。
const PATTERN_FAVORITE_COUNT: &str = r"(?<count>[\d,]+) times";
/// 新版本列表的时间文本（`added 2024-02-10 12:34`）。
const PATTERN_NEW_VERSION_DATETIME: &str = r"added (?<datetime>\d+-\d+-\d+ \d+:\d+)";

/// 冒犯性内容警告页特征。
const OFFENSIVE_STRING: &str =
    "<p>(And if you choose to ignore this warning, you lose all rights to complain about it in the future.)</p>";
/// "pining for the fjords"（删除流程中）页面特征。
const PINING_STRING: &str = "<p>This gallery is pining for the fjords.</p>";
/// 画廊不可用页特征。
const UNAVAILABLE_STRING: &str = "This gallery is unavailable";
/// 通用错误框特征（`class="d"` 容器内首段文本为错误说明；行间空白不敏感）。
const PATTERN_ERROR: &str = r#"<div class="d">\s*<p>([^<]+)</p>"#;

/// 画廊详情，由画廊详情页面解析获得。
///
/// `info` 字段为画廊基本信息（结构见 [`GalleryInfo`]），
/// 其余字段为详情页独有的会话参数、功能链接与统计数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryDetail {
    /// 画廊基本信息（gid/token/标题/分类/评分/页数等）。
    pub info: GalleryInfo,
    /// 会话 API uid（评分/评论投票用）。
    pub api_uid: i64,
    /// 会话 API key（评分/评论投票用）。
    pub api_key: String,
    /// 画廊的种子数量。
    pub torrent_count: i32,
    /// 画廊种子获取链接（`gallerytorrents.php` 弹窗地址）。
    pub torrent_url: String,
    /// 画廊存档获取链接（`archiver.php` 弹窗地址）。
    pub archive_url: String,
    /// 父画廊链接（无父子关系时为 `None`）。
    pub parent: Option<String>,
    /// 画廊是否可见（`Visible:` 行以 `Yes` 开头）。
    pub visible: bool,
    /// 画廊语言（已剥离 `TR` 翻译后缀）。
    pub language: String,
    /// 画廊文件总大小（页面渲染文本，如 `"105.2 MiB"`）。
    pub size: String,
    /// 画廊收藏数。
    pub favorite_count: i64,
    /// 画廊是否已被当前用户收藏（`#gdf` 文本不是 `Add to Favorites`）。
    pub is_favorited: bool,
    /// 画廊评分人数。
    pub rating_count: i64,
    /// 收藏夹名称（已收藏时为所在收藏夹的名称）。
    pub favorite_slot_name: Option<String>,
    /// 更新版本画廊列表（`#gnd`）。
    pub new_versions: Vec<GalleryNewVersion>,
    /// 画廊评论（见 [`GalleryComment`]）。
    pub comments: Vec<GalleryComment>,
    /// 画廊预览（新旧版式自动识别）。
    pub preview: GalleryPreview,
}

/// 画廊的新版本条目（同系列画廊的更新版本链接）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryNewVersion {
    /// 新版本画廊 ID。
    pub gid: i64,
    /// 新版本画廊令牌。
    pub token: String,
    /// 条目文本（站点渲染，通常为 "Parent: …" 类描述）。
    pub title: String,
    /// 更新时间（页面渲染文本按 UTC 解释，见 `parse_posted` 的时区说明）。
    pub update_at: DateTime<Utc>,
}

impl FromStr for GalleryNewVersion {
    type Err = Error;

    /// 从新版本条目的画廊 URL 解析 gid/token（title/update_at 需另行填充）。
    ///
    /// # Errors
    ///
    /// URL 不是画廊链接时返回 [`Error::Parse`]。
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let builder = GalleryBuilder::parse(s.to_string())?;
        Ok(Self {
            gid: builder.gid,
            token: builder.token,
            title: String::new(),
            update_at: DateTime::from_timestamp(0, 0).expect("constant timestamp"),
        })
    }
}

impl Default for GalleryDetail {
    fn default() -> Self {
        Self {
            info: GalleryInfo::default(),
            api_uid: -1,
            api_key: String::new(),
            torrent_count: 0,
            torrent_url: String::new(),
            archive_url: String::new(),
            parent: None,
            visible: false,
            language: String::new(),
            size: String::new(),
            favorite_count: 0,
            is_favorited: false,
            rating_count: 0,
            favorite_slot_name: None,
            new_versions: Vec::new(),
            comments: Vec::new(),
            preview: GalleryPreview::default(),
        }
    }
}

impl GalleryDetail {
    /// 从 HTML 解析画廊详情。
    ///
    /// 解析依次执行：特殊页嗅探 → 核心参数 → 元数据 → 新版本 →
    /// 标签 → 评论 → 预览。除评论与预览为尽力而为外，
    /// 其余环节失败会整体报错。
    ///
    /// # Errors
    ///
    /// - Offensive / Pining / 画廊不可用 / `class="d"` 错误框 → [`Error::Protocol`]；
    /// - 缺少 `var gid…` 内联脚本或关键 DOM 结构 → [`Error::Parse`]。
    pub fn parse(html: String) -> Result<Self, Error> {
        if crate::utils::regex::contains_phrase(&html, UNAVAILABLE_STRING) {
            return Err(Error::Protocol("gallery unavailable".into()));
        }
        if crate::utils::regex::contains_phrase(&html, OFFENSIVE_STRING)
            || crate::utils::regex::contains_phrase(&html, PINING_STRING)
        {
            return Err(Error::Protocol("offensive or pining".into()));
        }
        let error_re = crate::utils::regex::regex(PATTERN_ERROR)?;
        if let Some(caps) = error_re.captures(&html) {
            return Err(Error::Protocol(caps[1].to_string()));
        }

        let mut gallery_detail = Self::default();
        let d = Html::parse_document(&html);
        Self::parse_detail(&mut gallery_detail, &d, &html)?;
        gallery_detail.comments = GalleryComment::parse(&d).unwrap_or_default();
        gallery_detail.preview = GalleryPreview::parse(&d).unwrap_or_default();
        Ok(gallery_detail)
    }

    /// 解析画廊详情主体。
    fn parse_detail(gd: &mut Self, d: &Html, html: &str) -> Result<(), Error> {
        let r = crate::utils::regex::regex(PATTERN_DETAIL)?;
        let caps = r.captures(html).ok_or_else(|| {
            Error::parse(
                "gallery detail",
                "no inline script params",
                snippet(html, 512),
            )
        })?;
        gd.info.gid = parse_to::<i64>(&caps["gid"])?;
        gd.info.token = caps["token"].to_string();
        gd.api_uid = parse_to::<i64>(&caps["apiuid"])?;
        gd.api_key = caps["apikey"].to_string();

        // 功能链接：按锚点文本锚定，不依赖 gd5 内条目顺序
        for a in d.select(&selector("#gd5 a")?) {
            let Some(onclick) = a.value().attr("onclick") else {
                continue;
            };
            let text = text_content(a.text());
            let onclick_re = crate::utils::regex::regex(PATTERN_TORRENT_ONCLICK)?;
            if text.contains("Torrent Download") {
                if let Some(link_caps) = onclick_re.captures(onclick) {
                    gd.torrent_url = link_caps["link"].replace("&amp;", "&");
                }
                if let Some(count_caps) =
                    crate::utils::regex::regex(PATTERN_TORRENT_COUNT)?.captures(&text)
                {
                    gd.torrent_count = parse_to::<i32>(&count_caps["count"])?;
                }
            } else if text.contains("Archive Download") {
                if let Some(link_caps) = onclick_re.captures(onclick) {
                    gd.archive_url = link_caps["link"].replace("&amp;", "&");
                }
            }
        }

        // 封面：#gd1 内 div 的行内样式
        if let Some(thumb_ele) = d.select(&selector("#gd1 > div")?).next() {
            if let Some(style) = thumb_ele.value().attr("style") {
                gd.info.thumb = Self::parse_cover_style(style)?;
            }
        }

        // 标题（罗马字 / 日文）
        if let Some(title_ele) = d.select(&selector("#gn")?).next() {
            gd.info.title = text_content(title_ele.text()).trim().to_string();
        }
        if let Some(title_ele) = d.select(&selector("#gj")?).next() {
            gd.info.title_jpn = text_content(title_ele.text()).trim().to_string();
        }

        // 分类
        if let Some(cat_ele) = d.select(&selector("#gdc > div")?).next() {
            gd.info.category = Category::from(text_content(cat_ele.text()));
        }

        // 上传者（"(Disowned)" 视为无）
        if let Some(uploader_ele) = d.select(&selector("#gdn")?).next() {
            let text = text_content(uploader_ele.text());
            if !text.is_empty() && text != "(Disowned)" {
                gd.info.uploader = Some(text.trim().to_string());
            }
        }

        // 键值表 + 评分 + 收藏态
        Self::parse_detail_info(gd, d)?;

        if let Some(rating_ele) = d.select(&selector("#rating_count")?).next() {
            let text = text_content(rating_ele.text());
            if let Ok(value) = parse_to::<i64>(&text) {
                gd.rating_count = value;
            }
        }
        if let Some(rating_ele) = d.select(&selector("#rating_label")?).next() {
            let text = text_content(rating_ele.text());
            // "Not Yet Rated" 保持默认 -1；其余取最后一个空格后的数字
            if text != "Not Yet Rated" {
                if let Some(value) = text.split(' ').next_back() {
                    if let Ok(value) = parse_to::<f32>(value) {
                        gd.info.rating = value;
                    }
                }
            }
        }
        if let Some(favorite_ele) = d.select(&selector("#gdf")?).next() {
            let text = text_content(favorite_ele.text());
            if text.contains("Add to Favorites") {
                gd.is_favorited = false;
            } else if !text.is_empty() {
                gd.is_favorited = true;
                gd.favorite_slot_name = Some(text);
            }
        }

        Self::parse_new_version(gd, d)?;
        Self::parse_tag_groups(gd, d)?;

        Ok(())
    }

    /// 解析 `#gnd` 中的新版本画廊列表。
    ///
    /// 条目结构异常时跳过该条目（不视为解析失败）。
    fn parse_new_version(gd: &mut Self, d: &Html) -> Result<(), Error> {
        let s = selector("#gnd > a")?;
        let r = crate::utils::regex::regex(PATTERN_NEW_VERSION_DATETIME)?;
        for ele in d.select(&s) {
            let Some(href) = ele.attr("href") else {
                continue;
            };
            let title = text_content(ele.text());
            let Some(next) = ele.next_sibling() else {
                continue;
            };
            if !next.value().is_text() {
                continue;
            }
            let text = next
                .value()
                .as_text()
                .map(|t| t.trim().to_string())
                .unwrap_or_default();
            let Some(caps) = r.captures(&text) else {
                continue;
            };
            let Ok(update_at) = parse_posted(&caps["datetime"]) else {
                continue;
            };
            let Ok(mut new_version) = GalleryNewVersion::from_str(href) else {
                continue;
            };
            new_version.title = title;
            new_version.update_at = update_at;
            gd.new_versions.push(new_version);
        }
        Ok(())
    }

    /// 解析 `#gdd` 键值表（Posted/Parent/Visible/Language/File Size/Length/Favorited）。
    fn parse_detail_info(gd: &mut Self, d: &Html) -> Result<(), Error> {
        let s = selector("#gdd > table > tbody > tr")?;
        for tr in d.select(&s) {
            let Some(td1) = tr.first_element_child() else {
                continue;
            };
            let Some(td2) = td1.next_sibling_element() else {
                continue;
            };
            let key_text = text_content(td1.text());
            let value_text = text_content(td2.text());
            match key_text.as_str() {
                k if k.starts_with("Posted") => {
                    gd.info.posted = parse_posted(&value_text)?;
                    gd.info.posted_raw = Some(value_text);
                }
                k if k.starts_with("Parent") => {
                    if let Some(a) = td2.select(&selector("a")?).next() {
                        if let Some(href) = a.value().attr("href") {
                            gd.parent = Some(href.to_string());
                        }
                    }
                }
                k if k.starts_with("Visible") => {
                    gd.visible = value_text.trim().starts_with("Yes");
                }
                k if k.starts_with("Language") => {
                    // 剥离翻译后缀（如 "Chinese TR" → "Chinese"）
                    gd.language = value_text.trim().trim_end_matches(" TR").trim().to_string();
                }
                k if k.starts_with("File Size") => {
                    gd.size = value_text;
                }
                k if k.starts_with("Length") => {
                    let r = crate::utils::regex::regex(PATTERN_PAGES_TEXT)?;
                    if let Some(caps) = r.captures(&value_text) {
                        gd.info.pages = parse_to::<i64>(&caps["length"].replace(',', ""))?;
                    }
                }
                k if k.starts_with("Favorited") => {
                    gd.favorite_count = match value_text.trim() {
                        "Never" => 0,
                        "Once" => 1,
                        _ => {
                            let r = crate::utils::regex::regex(PATTERN_FAVORITE_COUNT)?;
                            match r.captures(&value_text) {
                                Some(caps) => parse_to::<i64>(&caps["count"].replace(',', ""))?,
                                None => 0,
                            }
                        }
                    };
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// 解析封面行内样式中的图片 URL。
    ///
    /// # Errors
    ///
    /// 样式中没有 `url(…)` 时返回 [`Error::Parse`](crate::error::Error::Parse)。
    fn parse_cover_style(style: &str) -> Result<String, Error> {
        let r = crate::utils::regex::regex(PATTERN_COVER)?;
        match r.captures(style) {
            Some(caps) => Ok(caps["link"].to_string()),
            None => Err(Error::parse("cover style", "no cover url", style)),
        }
    }

    /// 解析 `#taglist` 标签组。
    ///
    /// 每行的第一个单元格是命名空间（`female:`），第二个单元格内是
    /// 标签锚点；拼接后经 [`crate::dto::keyword::Keyword`] 解析。
    /// 单个标签解析异常被跳过，不影响其余标签。
    fn parse_tag_groups(gd: &mut Self, d: &Html) -> Result<(), Error> {
        let s = selector("#taglist tr")?;
        for tr in d.select(&s) {
            let Some(td1) = tr.first_element_child() else {
                continue;
            };
            let Some(td2) = td1.next_sibling_element() else {
                continue;
            };
            let tag_category = text_content(td1.text());
            let anchor_sel = selector("a")?;
            for a in td2.select(&anchor_sel) {
                let tag = text_content(a.text());
                // Keyword 解析永不失败（未知命名空间回退 Normal）
                let keyword = crate::dto::keyword::Keyword::from(format!("{tag_category}{tag}"));
                gd.info.tags.push(keyword);
            }
        }
        Ok(())
    }
}
