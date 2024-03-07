use std::str::FromStr;

use chrono::{DateTime, Utc};
use scraper::{Element, Html};
use serde::{Deserialize, Serialize};

use crate::{
    url::gallery::GalleryBuilder,
    utils::{
        regex::regex,
        scraper::{parse_posted, parse_to, selector, text_content},
    },
};

use super::{
    category::Category, gallery_comment::GalleryComment, gallery_info::GalleryInfo,
    gallery_preview::GalleryPreview, keyword::Keyword,
};

const PATTERN_ERROR: &'static str = r#"<div class="d">\n<p>([^<]+)</p>"#;
const PATTERN_DETAIL: &'static str = r#"var gid = (?<gid>\d+);\s+?var token = "(?<token>[a-f0-9]+)";\s+?var apiuid = (?<apiuid>-?\d+);\s+?var apikey = "(?<apikey>[a-f0-9]+)";"#;
#[allow(dead_code)]
const PATTERN_TORRENT: &'static str = r#"<a[^<>]*onclick="return popUp\('(?<link>[^']+)'[^)]+\)">Torrent Download \((<?count>\d+)\)</a>"#;
const PATTERN_TORRENT_ONCLICK: &'static str = r#"return popUp\('(?<link>[^']+)'[^)]+\)"#;
const PATTERN_TORRENT_COUNT: &'static str = r#"Torrent Download \((?<count>\d+)\)"#;
#[allow(dead_code)]
const PATTERN_ARCHIVE: &'static str =
    r#"<a[^<>]*onclick="return popUp\('([^']+)'[^)]+\)">Archive Download</a>"#;
const PATTERN_ARCHIVE_ONCLICK: &'static str = r#"return popUp\('(?<link>[^']+)'[^)]+\)"#;
const PATTERN_COVER: &'static str =
    r#"width:(?<width>\d+)px; height:(?<height>\d+)px.+?url\((?<link>.+?)\)"#;
#[allow(dead_code)]
const PATTERN_TAG_GROUP: &'static str =
    r#"<tr><td[^<>]+>([\w\s]+):</td><td>(?:<div[^<>]+><a[^<>]+>[\w\s]+</a></div>)+</td></tr>"#;
#[allow(dead_code)]
const PATTERN_TAG: &'static str = r#"<div[^<>]+><a[^<>]+>([\w\s]+)</a></div>"#;
#[allow(dead_code)]
const PATTERN_COMMENT: &'static str = r#"<div class=\"c3\">Posted on ([^<>]+) by: &nbsp; <a[^<>]+>([^<>]+)</a>.+?<div class=\"c6\"[^>]*>(.+?)</div><div class=\"c[78]\""#;
#[allow(dead_code)]
const PATTERN_PAGES: &'static str =
    r#"<tr><td[^<>]*>Length:</td><td[^<>]*>([\\d,]+) pages</td></tr>"#;
const PATTERN_PAGES_TEXT: &'static str = r"(?<length>\d+) pages";
const PATTERN_FAVORITE_COUNT: &'static str = r"(?<count>\d+) times";
const PATTERN_NEW_VERSION_DATETIME: &'static str = r"added (?<datetime>\d+-\d+-\d+ \d+:\d+)";
#[allow(dead_code)]
const PATTERN_PREVIEW_PAGES: &'static str =
    r#"<td[^>]+><a[^>]+>([\\d,]+)</a></td><td[^>]+>(?:<a[^>]+>)?&gt;(?:</a>)?</td>"#;
#[allow(dead_code)]
const PATTERN_NORMAL_PREVIEW: &'static str = r#"<div class=\"gdtm\"[^<>]*><div[^<>]*width:(\\d+)[^<>]*height:(\\d+)[^<>]*\\((.+?)\\)[^<>]*-(\\d+)px[^<>]*><a[^<>]*href=\"(.+?)\"[^<>]*><img alt=\"([\\d,]+)\""#;
#[allow(dead_code)]
const PATTERN_LARGE_PREVIEW: &'static str =
    r#"<div class=\"gdtl\".+?<a href=\"(.+?)\"><img alt=\"([\\d,]+)\".+?src=\"(.+?)\""#;

const OFFENSIVE_STRING: &'static str =
            "<p>(And if you choose to ignore this warning, you lose all rights to complain about it in the future.)</p>";
const PINING_STRING: &'static str = "<p>This gallery is pining for the fjords.</p>";

/// 画廊详情，由画廊详情页面解析获得
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryDetail {
    /// 画廊基本信息
    pub info: GalleryInfo,
    pub api_uid: i64,
    pub api_key: String,
    /// 画廊的种子数量
    pub torrent_count: i32,
    /// 画廊种子获取链接
    pub torrent_url: String,
    /// 画廊存档获取链接
    pub archive_url: String,
    /// 父画廊链接
    pub parent: Option<String>,
    /// 画廊是否可见
    pub visible: bool,
    /// 画廊语言
    pub language: String,
    /// 画廊文件总大小
    pub size: String,
    /// 画廊收藏数
    pub favorite_count: i64,
    /// 画廊是否已收藏
    pub is_favorited: bool,
    /// 画廊评分人数
    pub rating_count: i64,
    /// 收藏夹名称
    pub favorite_slot_name: Option<String>,
    /// 更新版本画廊列表
    pub new_versions: Vec<GalleryNewVersion>,
    /// 画廊评论
    pub comments: Vec<GalleryComment>,
    /// 画廊预览
    pub preview: GalleryPreview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryNewVersion {
    pub gid: i64,
    pub token: String,
    pub title: String,
    pub update_at: DateTime<Utc>,
}

impl FromStr for GalleryNewVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let builder = GalleryBuilder::parse(s.into())?;
        Ok(Self {
            gid: builder.gid,
            token: builder.token,
            title: String::new(),
            update_at: DateTime::from_timestamp(0, 0).unwrap(),
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
    /// 从 HTML 解析画廊详情
    pub fn parse(html: String) -> Result<Self, String> {
        if html.contains(OFFENSIVE_STRING) || html.contains(PINING_STRING) {
            return Err(format!(
                "Failed to parse gallery detail: {}",
                "Offensive or pining."
            ));
        }
        let r = regex(PATTERN_ERROR)?;
        if r.is_match(&html) {
            return Err(format!("Failed to parse gallery detail: {}", "Page error."));
        }

        let mut gallery_detail = Self::default();
        let d = Html::parse_document(&html);
        Self::parse_detail(&mut gallery_detail, &d, html)?;
        let comments = GalleryComment::parse(&d)?;
        gallery_detail.comments = comments;
        Ok(gallery_detail)
    }

    /// 解析画廊详情
    fn parse_detail(gd: &mut Self, d: &Html, html: String) -> Result<(), String> {
        let r = regex(PATTERN_DETAIL)?;
        let caps = match r.captures(&html) {
            Some(caps) => caps,
            None => return Err(format!("Failed to parse gallery detail: {}", "No detail.")),
        };
        // GID
        let gid = parse_to::<i64>(&caps["gid"])?;
        gd.info.gid = gid;
        // Token
        let token = caps["token"].to_string();
        gd.info.token = token;
        // API UID
        let api_uid = parse_to::<i64>(&caps["apiuid"])?;
        gd.api_uid = api_uid;
        // API KEY
        let api_key = caps["apikey"].to_string();
        gd.api_key = api_key;

        // Torrent
        let s = selector("#gd5 > p:nth-child(3) > a")?;
        if let Some(torrent_ele) = d.select(&s).next() {
            if let Some(link) = torrent_ele.attr("onclick") {
                let r = regex(PATTERN_TORRENT_ONCLICK)?;
                if let Some(caps) = r.captures(&link) {
                    let link = caps["link"].to_string();
                    gd.torrent_url = link;
                }
            }
            let text = text_content(torrent_ele.text());
            let r = regex(PATTERN_TORRENT_COUNT)?;
            if let Some(caps) = r.captures(&text) {
                let torrent_count = parse_to::<i32>(&caps["count"])?;
                gd.torrent_count = torrent_count;
            }
        }

        //  Archive
        let s = selector("#gd5 > p:nth-child(2) > a")?;
        if let Some(archive_ele) = d.select(&s).next() {
            if let Some(link) = archive_ele.attr("onclick") {
                let r = regex(PATTERN_ARCHIVE_ONCLICK)?;
                if let Some(caps) = r.captures(&link) {
                    let link = caps["link"].to_string();
                    gd.archive_url = link;
                }
            }
        }

        // Thumb
        let s = selector("#gd1 > div")?;
        if let Some(thumb_ele) = d.select(&s).next() {
            if let Some(style) = thumb_ele.attr("style") {
                gd.info.thumb = Self::parse_cover_style(style)?;
            }
        }

        // Title
        let s = selector("#gn")?;
        if let Some(title_ele) = d.select(&s).next() {
            let text = text_content(title_ele.text());
            gd.info.title = text.trim().to_string();
        }

        // Title Jpn
        let s = selector("#gj")?;
        if let Some(title_ele) = d.select(&s).next() {
            let text = text_content(title_ele.text());
            gd.info.title_jpn = text.trim().to_string();
        }

        // Category
        let s = selector("#gdc > div")?;
        if let Some(cat_ele) = d.select(&s).next() {
            let text = text_content(cat_ele.text());
            gd.info.category = Category::from(text);
        }

        // Uploader
        let s = selector("#gdn")?;
        if let Some(uploader_ele) = d.select(&s).next() {
            let text = text_content(uploader_ele.text());
            if !text.is_empty() && text != "(Disowned)" {
                gd.info.uploader = Some(text.trim().to_string());
            }
        }

        // GalleryInfo
        Self::parse_detail_info(gd, &d)?;

        // Rating Count
        let s = selector("#rating_count")?;
        if let Some(rating_ele) = d.select(&s).next() {
            let text = text_content(rating_ele.text());
            if let Ok(value) = parse_to::<i64>(&text) {
                gd.rating_count = value;
            }
        }

        // Rating
        let s = selector("#rating_label")?;
        if let Some(rating_ele) = d.select(&s).next() {
            let text = text_content(rating_ele.text());
            if let Some(value) = text.split(" ").last() {
                if let Ok(value) = parse_to::<f32>(&value) {
                    gd.info.rating = value;
                }
            }
        }

        // isFavorited
        let s = selector("#gdf")?;
        if let Some(favorite_ele) = d.select(&s).next() {
            let text = text_content(favorite_ele.text());
            if text.contains("Add to Favorites") {
                gd.is_favorited = false;
            } else {
                gd.is_favorited = true;
                gd.favorite_slot_name = Some(text);
            }
        }

        // 解析画廊新版本
        Self::parse_new_version(gd, &d)?;

        // 解析画廊标签
        Self::parse_tag_groups(gd, &d)?;

        Ok(())
    }

    /// 解析当前画廊是否有新版本
    fn parse_new_version(gd: &mut Self, d: &Html) -> Result<(), String> {
        let s = selector("#gnd > a")?;
        let r = regex(PATTERN_NEW_VERSION_DATETIME)?;
        for ele in d.select(&s) {
            if let Some(href) = ele.attr("href") {
                let title = text_content(ele.text());
                if let Some(next) = ele.next_sibling() {
                    if next.value().is_text() {
                        let text = next.value().as_text().unwrap();
                        let text = text.trim().to_string();
                        if let Some(caps) = r.captures(&text) {
                            let datetime = caps["datetime"].to_string();
                            let datetime = parse_posted(&datetime)?;
                            let mut new_version = GalleryNewVersion::from_str(href)?;
                            new_version.title = title;
                            new_version.update_at = datetime;
                            gd.new_versions.push(new_version);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// 解析画廊元数据
    fn parse_detail_info(gd: &mut Self, d: &Html) -> Result<(), String> {
        // 选择器：获取表格各行
        let s = selector("#gdd > table > tbody > tr")?;
        let selected = d.select(&s);
        for tr in selected {
            if let Some(td1) = tr.first_element_child() {
                if let Some(td2) = td1.next_sibling_element() {
                    // 获取键文本
                    let key_text = text_content(td1.text());
                    // 获取值文本
                    let value_text = text_content(td2.text());
                    match key_text.as_str() {
                        s if s.starts_with("Posted") => {
                            // 解析发布日期
                            let posted = parse_posted(&value_text)?;
                            // 写入发布日期
                            gd.info.posted = posted;
                        }
                        s if s.starts_with("Parent") => {
                            let s = selector("a")?;
                            if let Some(a) = td2.select(&s).next() {
                                if let Some(href) = a.attr("href") {
                                    // 写入父链接
                                    gd.parent = Some(href.into());
                                }
                            }
                        }
                        s if s.starts_with("Visible") => {
                            // 设置可见性
                            gd.visible = match value_text.trim() {
                                s if s.starts_with("Yes") => true,
                                _ => false,
                            }
                        }
                        s if s.starts_with("Language") => {
                            // 设置语言
                            gd.language = value_text.replace("TR", "").trim().into();
                        }
                        s if s.starts_with("File Size") => {
                            // 写入文件大小
                            gd.size = value_text;
                        }
                        s if s.starts_with("Length") => {
                            let r = regex(PATTERN_PAGES_TEXT)?;
                            if let Some(caps) = r.captures(&value_text) {
                                let pages = parse_to::<i64>(&caps["length"])?;
                                gd.info.pages = pages; // 设置页面数
                            }
                        }
                        s if s.starts_with("Favorited") => {
                            // 设置收藏数
                            gd.favorite_count = match value_text.trim() {
                                "Never" => 0,
                                "Once" => 1,
                                _ => {
                                    let r = regex(PATTERN_FAVORITE_COUNT)?;
                                    match r.captures(&value_text) {
                                        Some(caps) => parse_to::<i64>(&caps["count"])?,
                                        None => {
                                            return Err(format!(
                                                "Failed to parse favorite count: {}",
                                                "No count."
                                            ))
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    /// 解析封面样式中的链接
    fn parse_cover_style(style: &str) -> Result<String, String> {
        let r = regex(PATTERN_COVER)?;
        match r.captures(style) {
            Some(caps) => {
                let cover = caps["link"].to_string();
                Ok(cover)
            }
            None => Err(format!("Failed to parse cover style: {}", "No cover.")),
        }
    }

    /// 解析标签组
    fn parse_tag_groups(gd: &mut Self, d: &Html) -> Result<(), String> {
        // 选择器：选择包含标签组的tr元素
        let s = selector("#taglist tr")?;
        // 遍历每个tr元素
        for tr in d.select(&s) {
            // 获取第一个td元素
            if let Some(td1) = tr.first_element_child() {
                // 获取下一个兄弟元素
                if let Some(td2) = td1.next_sibling_element() {
                    // 获取标签组类别
                    let tag_category = text_content(td1.text());
                    // 选择器：选择a元素
                    let s = selector("a")?;
                    // 遍历每个a元素
                    for a in td2.select(&s) {
                        // 获取标签
                        let tag = text_content(a.text());
                        // 创建关键词
                        let keyword = Keyword::from_str(&format!("{}{}", tag_category, tag))?;
                        // 将关键词添加到信息的标签列表中
                        gd.info.tags.push(keyword);
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    use super::GalleryDetail;

    #[test]
    fn test_parse() -> Result<(), Box<dyn std::error::Error>> {
        let mut cwd = std::env::current_dir().unwrap();
        cwd.push("../../samples/gallery_old.html");
        let mut file = match File::open(cwd) {
            Ok(file) => file,
            Err(err) => panic!("Failed to open file: {}", err),
        };
        let html = {
            let mut buf = String::new();
            match file.read_to_string(&mut buf) {
                Ok(_) => buf,
                Err(err) => panic!("Failed to read file: {}", err),
            }
        };
        match GalleryDetail::parse(html) {
            Ok(result) => {
                println!("{:?}", result);
                let json = serde_json::to_string(&result)?;
                println!("{}", json);
                println!("{:?}", result.comments[1]);
            }
            Err(err) => panic!("Failed to parse search result: {}", err),
        }
        Ok(())
    }
}
