use scraper::{Element, Html};
use serde::{Deserialize, Serialize};

use crate::utils::{
    regex::regex,
    scraper::{parse_posted, parse_to, selector, text_content},
};

use super::{category::Category, gallery_info::GalleryInfo};

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
const PATTERN_TAG_GROUP: &'static str =
    r#"<tr><td[^<>]+>([\w\s]+):</td><td>(?:<div[^<>]+><a[^<>]+>[\w\s]+</a></div>)+</td></tr>"#;
const PATTERN_TAG: &'static str = r#"<div[^<>]+><a[^<>]+>([\w\s]+)</a></div>"#;
const PATTERN_COMMENT: &'static str = r#"<div class=\"c3\">Posted on ([^<>]+) by: &nbsp; <a[^<>]+>([^<>]+)</a>.+?<div class=\"c6\"[^>]*>(.+?)</div><div class=\"c[78]\""#;
#[allow(dead_code)]
const PATTERN_PAGES: &'static str =
    r#"<tr><td[^<>]*>Length:</td><td[^<>]*>([\\d,]+) pages</td></tr>"#;
const PATTERN_PAGES_TEXT: &'static str = r"(?<length>\d+) pages";
const PATTERN_FAVORITE_COUNT: &'static str = r"(?<count>\d+) times";
const PATTERN_RATING: &'static str = r"";
const PATTERN_PREVIEW_PAGES: &'static str =
    r#"<td[^>]+><a[^>]+>([\\d,]+)</a></td><td[^>]+>(?:<a[^>]+>)?&gt;(?:</a>)?</td>"#;
const PATTERN_NORMAL_PREVIEW: &'static str = r#"<div class=\"gdtm\"[^<>]*><div[^<>]*width:(\\d+)[^<>]*height:(\\d+)[^<>]*\\((.+?)\\)[^<>]*-(\\d+)px[^<>]*><a[^<>]*href=\"(.+?)\"[^<>]*><img alt=\"([\\d,]+)\""#;
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
    pub spider_info_pages: i64,
    /// 画廊收藏数
    pub favorite_count: i64,
    /// 画廊是否已收藏
    pub is_favorited: bool,
    /// 画廊评分人数
    pub rating_count: i64,
    /// 收藏夹名称
    pub favorite_slot_name: Option<String>,
    // public GalleryTagGroup[] tags;
    // public GalleryCommentList comments;
    // public int previewPages;
    // public int SpiderInfoPreviewPages;
    // public PreviewSet previewSet;
    // public PreviewSet SpiderInfoPreviewSet;
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
            spider_info_pages: 0,
            favorite_count: 0,
            is_favorited: false,
            rating_count: 0,
            favorite_slot_name: None,
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
        Self::parse_detail(&mut gallery_detail, d, html)?;
        Ok(gallery_detail)
    }

    /// 解析画廊详情
    fn parse_detail(gd: &mut Self, d: Html, html: String) -> Result<(), String> {
        let r = regex(PATTERN_DETAIL)?;
        let caps = match r.captures(&html) {
            Some(caps) => caps,
            None => return Err(format!("Failed to parse gallery detail: {}", "No detail.")),
        };
        // GID
        let gid = match parse_to::<i64>(&caps["gid"]) {
            Ok(gid) => gid,
            Err(err) => panic!("Failed to parse gallery detail: {}", err),
        };
        gd.info.gid = gid;
        // Token
        let token = caps["token"].to_string();
        gd.info.token = token;
        // API UID
        let api_uid = match parse_to::<i64>(&caps["apiuid"]) {
            Ok(api_uid) => api_uid,
            Err(err) => panic!("Failed to parse gallery detail: {}", err),
        };
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
            let text: Vec<String> = text_content(torrent_ele.text());
            let text = text.join("");
            let r = regex(PATTERN_TORRENT_COUNT)?;
            if let Some(caps) = r.captures(&text) {
                let torrent_count = match parse_to::<i32>(&caps["count"]) {
                    Ok(torrent_count) => torrent_count,
                    Err(_) => panic!("Failed to parse gallery detail: {}", "No torrent count."),
                };
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
            let text = text.join("");
            gd.info.title = text.trim().to_string();
        }

        // Title Jpn
        let s = selector("#gj")?;
        if let Some(title_ele) = d.select(&s).next() {
            let text = text_content(title_ele.text());
            let text = text.join("");
            gd.info.title_jpn = text.trim().to_string();
        }

        // Category
        let s = selector("#gdc > div")?;
        if let Some(cat_ele) = d.select(&s).next() {
            let text = text_content(cat_ele.text());
            let text = text.join("");
            gd.info.category = Category::from(text);
        }

        // Uploader
        let s = selector("#gdn")?;
        if let Some(uploader_ele) = d.select(&s).next() {
            let text = text_content(uploader_ele.text());
            let text = text.join("");
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
            let text = text.join("");
            if let Ok(value) = parse_to::<i64>(&text) {
                gd.rating_count = value;
            }
        }

        // Rating
        let s = selector("#rating_label")?;
        if let Some(rating_ele) = d.select(&s).next() {
            let text = text_content(rating_ele.text());
            let text = text.join("");
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
            let text = text.join("");
            if text.contains("Add to Favorites") {
                gd.is_favorited = false;
            } else {
                gd.is_favorited = true;
                gd.favorite_slot_name = Some(text);
            }
        }

        Ok(())
    }

    /// 解析画廊元数据
    fn parse_detail_info(gd: &mut Self, d: &Html) -> Result<(), String> {
        let s = selector("#gdd > table > tbody > tr")?; // 选择器
        let selected = d.select(&s);
        for tr in selected {
            if let Some(td1) = tr.first_element_child() {
                if let Some(td2) = td1.next_sibling_element() {
                    let key_text = text_content(td1.text()); // 获取键文本
                    let key_text = key_text.join("").trim().to_string(); // 清理键文本
                    let value_text = text_content(td2.text()); // 获取值文本
                    let value_text = value_text.join("").trim().to_string(); // 清理值文本
                    match key_text.as_str() {
                        s if s.starts_with("Posted") => {
                            let posted = parse_posted(&value_text)?; // 解析发布日期
                            gd.info.posted = posted; // 写入发布日期
                        }
                        s if s.starts_with("Parent") => {
                            let s = selector("a")?; // 选择器
                            if let Some(a) = td2.select(&s).next() {
                                if let Some(href) = a.attr("href") {
                                    gd.parent = Some(href.into()); // 写入父链接
                                }
                            }
                        }
                        s if s.starts_with("Visible") => {
                            gd.visible = match value_text.trim() {
                                s if s.starts_with("Yes") => true, // 设置可见性
                                _ => false,
                            }
                        }
                        s if s.starts_with("Language") => {
                            gd.language = value_text.replace("TR", ""); // 设置语言
                        }
                        s if s.starts_with("File Size") => {
                            gd.size = value_text; // 设置文件大小
                        }
                        s if s.starts_with("Length") => {
                            let r = regex(PATTERN_PAGES_TEXT)?; // 正则表达式
                            if let Some(caps) = r.captures(&value_text) {
                                let pages = match parse_to::<i64>(&caps["length"]) {
                                    Ok(pages) => pages, // 解析页面数
                                    Err(_) => panic!("Failed to parse pages: {}", "No pages."),
                                };
                                gd.info.pages = pages; // 设置页面数
                            }
                        }
                        s if s.starts_with("Favorited") => {
                            gd.favorite_count = match value_text.trim() {
                                "Never" => 0, // 设置收藏数
                                "Once" => 1,
                                _ => {
                                    let r = regex(PATTERN_FAVORITE_COUNT)?; // 正则表达式
                                    match r.captures(&value_text) {
                                        Some(caps) => match caps["count"].parse::<i64>() {
                                            Ok(count) => count, // 解析收藏数
                                            Err(_) => panic!(
                                                "Failed to parse favorite count: {}",
                                                "No count."
                                            ),
                                        },
                                        None => panic!(
                                            "Failed to parse favorite count: {}",
                                            "No count."
                                        ),
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
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    use super::GalleryDetail;

    #[test]
    fn test_parse() {
        let mut cwd = std::env::current_dir().unwrap();
        cwd.push("../../samples/gallery_faved.html");
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
            }
            Err(err) => panic!("Failed to parse search result: {}", err),
        }
    }
}
