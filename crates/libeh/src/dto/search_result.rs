use std::str::FromStr;

use scraper::{ElementRef, Html};
use serde::{Deserialize, Serialize};

use crate::{
    dto::{gallery::category::Category, keyword::Keyword},
    url::gallery::GalleryBuilder,
    utils::{
        regex::regex,
        scraper::{
            parse_favorite_slot, parse_posted, parse_rating, parse_to, selector, text_content,
        },
    },
};

use super::gallery::info::GalleryInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub pages: isize,
    pub next_page: isize,
    pub first_href: Option<String>,
    pub prev_href: Option<String>,
    pub next_href: Option<String>,
    pub last_href: Option<String>,
    pub no_watched_tags: bool,
    pub gallery_info_list: Vec<GalleryInfo>,
}

impl Default for SearchResult {
    fn default() -> Self {
        SearchResult {
            pages: -1,
            next_page: -1,
            first_href: None,
            prev_href: None,
            next_href: None,
            last_href: None,
            no_watched_tags: false,
            gallery_info_list: vec![],
        }
    }
}

impl SearchResult {
    pub fn parse(html: String) -> Result<Self, String> {
        let mut search_result = SearchResult::default();
        let d = Html::parse_document(&html);

        let selector_search_nav = selector(".searchnav")?;
        let mut result_search_nav = d.select(&selector_search_nav);
        let search_nav = if let Some(result) = result_search_nav.next() {
            result
        } else {
            return Err(format!("Failed to parse search nav."));
        };

        let s = selector("#ufirst")?;
        if let Some(result) = search_nav.select(&s).next() {
            search_result.first_href = match result.value().attr("href") {
                Some(href) => Some(href.to_string()),
                None => None,
            };
        }

        let s = selector("#uprev")?;
        if let Some(result) = search_nav.select(&s).next() {
            search_result.prev_href = match result.value().attr("href") {
                Some(href) => Some(href.to_string()),
                None => None,
            };
        }

        let s = selector("#unext")?;
        if let Some(result) = search_nav.select(&s).next() {
            search_result.next_href = match result.value().attr("href") {
                Some(href) => Some(href.to_string()),
                None => None,
            };
        }

        let s = selector("#ulast")?;
        if let Some(result) = search_nav.select(&s).next() {
            search_result.last_href = match result.value().attr("href") {
                Some(href) => Some(href.to_string()),
                None => None,
            };
        }

        let s = selector("table.itg")?;
        let table = match d.select(&s).next() {
            Some(table) => table,
            None => return Err(format!("Failed to parse search result: {}", "No table.")),
        };

        // let mut list: Vec<GalleryInfo> = vec![];
        let s = selector("tr")?;
        for tr in table.select(&s) {
            match Self::parse_gallery_info(tr) {
                Ok(gallery_info) => search_result.gallery_info_list.push(gallery_info),
                Err(err) => println!("Failed to parse gallery info: {}", err),
            }
        }
        Ok(search_result)
    }

    fn parse_gallery_info(tr: ElementRef) -> Result<GalleryInfo, String> {
        let mut gi = GalleryInfo::default();
        // 提取标题
        let s = selector(".glname")?;
        let glname = match tr.select(&s).next() {
            Some(element) => element,
            None => {
                return Err(format!(
                    "Failed to parse gallery title: {}",
                    "No valid title."
                ))
            }
        };

        let s = selector(".glink")?;
        let glink = match tr.select(&s).next() {
            Some(element) => element,
            None => return Err(format!("")),
        };
        let text = text_content(glink.text());
        gi.title = text;
        if gi.title.is_empty() {
            return Err(format!(
                "Failed to parse gallery title: {}",
                "Title is empty."
            ));
        }

        // 提取画廊id和token
        let s = selector("a")?;
        let a = match glname.select(&s).next() {
            Some(element) => element.value(),
            None => {
                let Some(parent) = glname.parent() else {
                    return Err(format!("Failed to parse gallery info: {}", "No link."));
                };
                let parent = parent.value();
                if !parent.is_element() {
                    return Err(format!("Failed to parse gallery info: {}", "No link."));
                };
                let parent = parent.as_element();
                let Some(parent) = parent else {
                    return Err(format!("Failed to parse gallery info: {}", "No link."));
                };
                if parent.name() == "a" {
                    parent
                } else {
                    return Err(format!("Failed to parse gallery info: {}", "No link."));
                }
            }
        };
        let href = a.attr("href");
        if let Some(href) = href {
            let result = GalleryBuilder::parse(href.into())?;
            gi.gid = result.gid;
            gi.token = result.token;
        } else {
            return Err(format!("Failed to parse gallery info: {}", "No link."));
        }

        // 提取 tags
        let s = selector("div > div.gt")?;
        let eles = glname.select(&s);
        for ele in eles {
            if let Some(tag) = ele.attr("title") {
                let tag = Keyword::from_str(tag)?;
                gi.tags.push(tag);
            }
        }

        // 提取画廊分类
        let s = selector(".cn")?;
        let cn = match tr.select(&s).next() {
            Some(element) => element,
            None => return Err(format!("Failed to parse gallery info: {}", "No category.")),
        };
        let text = text_content(cn.text());
        let category = Category::from(text);
        gi.category = category;
        // println!("Element: {:?}", category);

        // 提取画廊封面
        let s = selector(".glthumb")?;
        let glthumb = match tr.select(&s).next() {
            Some(element) => element,
            None => {
                return Err(format!(
                    "Failed to parse gallery info: {}",
                    "No thumb class element."
                ))
            }
        };
        let s = selector("div:nth-child(1) > img")?;
        let glthumb_img = match glthumb.select(&s).next() {
            Some(element) => element,
            None => {
                return Err(format!(
                    "Failed to parse gallery info: {}",
                    "No thumb img element."
                ))
            }
        };
        let data_src = glthumb_img.attr("data-src");
        let src = match data_src {
            Some(src) => src,
            None => match glthumb_img.attr("src") {
                Some(src) => src,
                None => return Err(format!("Failed to parse gallery info: {}", "No thumb src.")),
            },
        };
        gi.thumb = src.to_string();

        // 提取画廊页数
        let s = selector(".ir + div")?;
        let pages = match tr.select(&s).next() {
            Some(element) => element,
            None => return Err(format!("Failed to parse gallery info: {}", "No pages.")),
        };
        let text = text_content(pages.text());
        let r = regex(r"(?<page>\d+) pages?")?;
        match r.captures(&text) {
            Some(caps) => {
                let page = {
                    let Ok(page) = parse_to::<i64>(&caps["page"]) else {
                        return Err(format!(
                            "Failed to parse gallery info: {}",
                            "Page parse error."
                        ));
                    };
                    page
                };
                gi.pages = page;
            }
            None => return Err(format!("Failed to parse gallery info: {}", "No page.")),
        }

        // 提取上传时间与收藏信息
        let s = selector(&String::from_iter(vec![
            "#posted_",
            gi.gid.to_string().as_str(),
        ]))?;
        let posted = match tr.select(&s).next() {
            Some(element) => element,
            None => return Err(format!("Failed to parse gallery info: {}", "No posted.")),
        };
        let posted_text = text_content(posted.text());
        gi.posted = parse_posted(&posted_text)?;
        if let Some(style) = posted.attr("style") {
            gi.favorite_slot = match parse_favorite_slot(style) {
                Ok(slot) => slot,
                Err(_) => -1,
            };
        }

        // 提取评分
        let s = selector(".ir")?;
        let ir = match tr.select(&s).next() {
            Some(element) => element,
            None => return Err(format!("Failed to parse gallery info: {}", "No ir.")),
        };
        if let Some(style) = ir.attr("style") {
            gi.rating = parse_rating(style)?;
        }

        // 提取上传者
        let s = selector(".glhide > div:nth-child(1)")?;
        let glhide = match tr.select(&s).next() {
            Some(element) => element,
            None => return Err(format!("Failed to parse gallery info: {}", "No glhide.")),
        };
        let uploader = text_content(glhide.text());
        if uploader.ne("(Disowned)") {
            gi.uploader = Some(uploader);
        }

        Ok(gi)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;

    use crate::dto::search_result::SearchResult;

    #[test]
    fn test_parse_search_result() {
        let mut cwd = std::env::current_dir().unwrap();
        cwd.push("../../samples/search.html");
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
        match SearchResult::parse(html) {
            Ok(result) => {
                for gallery_info in result.gallery_info_list {
                    println!("GalleryInfo: {:?}", gallery_info);
                }
            }
            Err(err) => panic!("Failed to parse search result: {}", err),
        }
        // assert_eq!(result, SearchResult {});
    }
}
