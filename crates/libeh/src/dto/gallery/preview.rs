//! 画廊预览信息（详情页 `#gdt` 区域）的解析器。
//!
//! 站点存在两种预览版式，本解析器都支持：
//!
//! - **新版（大图/gt200、小图/gt100）**：`#gdt` 内直接是
//!   `<a href="页面链接"><div title="Page N: 文件名" style="… url(缩略图) X Y …"></div></a>`；
//! - **旧版（雪碧图/gdtm）**：`#gdt` 内是 `<div class="gdtm"><div style="… url(雪碧图) -Xpx -Ypx…"><a href="页面链接">`，
//!   缩略图是一张雪碧图，`x/y` 偏移定位当前页的小图。
//!
//! 两种版式统一产出 [`GalleryPreviewPage`](crate::dto::gallery::preview::GalleryPreviewPage)：新版式偏移为 `(0, 0)`。
//!
//! # 示例
//!
//! ```rust,no_run
//! # use libeh::dto::gallery::preview::GalleryPreview;
//! # use scraper::Html;
//! # fn demo(html: &str) -> Result<(), libeh::error::Error> {
//! let d = Html::parse_document(html);
//! let preview = GalleryPreview::parse(&d)?;
//! println!("total {} images across {} preview pages", preview.total, preview.total_set);
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::utils::{
    regex::regex,
    scraper::{parse_to, selector, text_content},
};

/// `Showing 1 - 20 of 838 images` 中的总量。
const PATTERN_TOTAL_PAGES: &str =
    r"Showing ((\d+)(,\d+)*) - ((\d+)(,\d+)*) of (?<total>(\d+)(,\d+)*) images";
/// 行内样式中的雪碧图 URL 与像素偏移。
const PATTERN_STYLE: &str =
    r"background:transparent url\((?<url>[^\(\)]+)\) (?<x>-?\d+)(px)? (?<y>-?\d+)(px)?";
/// 新版式 `title="Page N: 文件名"` 中的页号。
const PATTERN_PAGE_TITLE: &str = r"Page (?<page>\d+):";

/// 画廊预览信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryPreview {
    /// 画廊中总共的图片数量（`Showing 1 - 20 of 838 images` 的 838）。
    pub total: i64,
    /// 画廊预览的总分页数量（预览分页表 `ptt` 的总页数）。
    pub total_set: i64,
    /// 当前预览页包含的分页条目。
    pub pages: Vec<GalleryPreviewPage>,
}

/// 单个预览条目（一页的小图）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryPreviewPage {
    /// 雪碧图上的 X 轴偏移（像素；新版式为 0）。
    pub offset_x: i64,
    /// 雪碧图上的 Y 轴偏移（像素；新版式为 0）。
    pub offset_y: i64,
    /// 页号（1 基；新版式从 `title` 提取，旧版式按出现顺序）。
    pub page: i64,
    /// 预览图片的 URL（新版式为大图，旧版式为雪碧图 + 偏移）。
    pub url: String,
    /// 该预览指向的页面链接（`/s/{pToken}/{gid}-{页号}`）。
    pub link: String,
}

impl Default for GalleryPreview {
    /// 返回空预览（total/total_set 为 0，无条目）。
    fn default() -> Self {
        GalleryPreview {
            total: 0,
            total_set: 0,
            pages: vec![],
        }
    }
}

impl GalleryPreview {
    /// 解析详情页的预览区域。
    ///
    /// # Errors
    ///
    /// 缺少 `div.gtb > p.gpc`（图片总量）或 `table.ptt`（预览分页数）时
    /// 返回 [`Error::Parse`]；`#gdt` 区域缺失时条目列表为空（不算失败）。
    pub fn parse(d: &scraper::Html) -> Result<Self, Error> {
        let total = Self::parse_total_page_count(d)?;
        let total_set = Self::parse_total_preview_set(d)?;
        let pages = Self::parse_preview_pages(d).unwrap_or_default();

        Ok(GalleryPreview {
            total,
            total_set,
            pages,
        })
    }

    /// 解析画廊图片总数。
    ///
    /// # Errors
    ///
    /// 缺少 `p.gpc` 或文本不符合 `Showing … of N images` 时返回 [`Error::Parse`]。
    pub fn parse_total_page_count(d: &scraper::Html) -> Result<i64, Error> {
        let r = regex(PATTERN_TOTAL_PAGES)?;
        let s = selector("div.gtb > p.gpc")?;
        let td = d.select(&s).next().ok_or_else(|| {
            Error::parse_msg("total preview pages", "no gpc element", String::new())
        })?;
        let text = text_content(td.text());
        let caps = match r.captures(&text) {
            Some(caps) => caps,
            None => {
                return Err(Error::parse(
                    "total preview pages",
                    "no total in text",
                    text,
                ))
            }
        };
        parse_to::<i64>(&caps["total"].replace(',', ""))
    }

    /// 解析画廊预览的总分页数量（`ptt` 表倒数第二个单元格）。
    ///
    /// # Errors
    ///
    /// 缺少预览分页表时返回 [`Error::Parse`]。
    pub fn parse_total_preview_set(d: &scraper::Html) -> Result<i64, Error> {
        let s = selector("div.gtb > table.ptt tr > td:nth-last-child(2)")?;
        match d.select(&s).next() {
            Some(td) => parse_to::<i64>(&text_content(td.text())),
            None => Err(Error::parse_msg(
                "total preview pages",
                "no preview pagination",
                String::new(),
            )),
        }
    }

    /// 解析预览条目列表（自动识别新版式与旧版雪碧图版式）。
    ///
    /// # Errors
    ///
    /// 仅当行内样式存在但无法解析偏移数字时返回 [`Error::Parse`]；
    /// 结构不匹配的条目（无 style/无链接）被跳过。
    pub fn parse_preview_pages(d: &scraper::Html) -> Result<Vec<GalleryPreviewPage>, Error> {
        let mut list = Vec::new();

        // 旧版雪碧图：#gdt > div.gdtm > div[style]
        if let Ok(s_div) = selector("#gdt > div.gdtm > div") {
            let r = regex(PATTERN_STYLE)?;
            let s_a = selector("a")?;
            for (page, div) in d.select(&s_div).enumerate() {
                let page = page as i64 + 1;
                let Some(style) = div.attr("style") else {
                    continue;
                };
                let Some(caps) = r.captures(style) else {
                    continue;
                };
                let url = caps["url"].to_string();
                let offset_x = parse_to::<i64>(&caps["x"])?;
                let offset_y = parse_to::<i64>(&caps["y"])?;
                let Some(link) = div.select(&s_a).next() else {
                    continue;
                };
                let Some(href) = link.value().attr("href") else {
                    continue;
                };
                list.push(GalleryPreviewPage {
                    page,
                    offset_x,
                    offset_y,
                    url,
                    link: href.to_string(),
                });
            }
        }

        // 新版式：#gdt > a > div[title="Page N: …"][style]
        if list.is_empty() {
            if let (Ok(s_a), Ok(s_div)) = (selector("#gdt > a"), selector("div")) {
                let r_style = regex(PATTERN_STYLE)?;
                let r_title = regex(PATTERN_PAGE_TITLE)?;
                for a in d.select(&s_a) {
                    let Some(link) = a.value().attr("href") else {
                        continue;
                    };
                    let Some(div) = a.select(&s_div).next() else {
                        continue;
                    };
                    let Some(style) = div.attr("style") else {
                        continue;
                    };
                    let Some(caps) = r_style.captures(style) else {
                        continue;
                    };
                    let url = caps["url"].to_string();
                    // 页号优先取 title 里的 "Page N:"，取不到按出现顺序编号
                    let page = div
                        .attr("title")
                        .and_then(|t| r_title.captures(t))
                        .and_then(|c| c["page"].parse::<i64>().ok())
                        .unwrap_or(list.len() as i64 + 1);
                    list.push(GalleryPreviewPage {
                        page,
                        offset_x: parse_to::<i64>(&caps["x"]).unwrap_or(0),
                        offset_y: parse_to::<i64>(&caps["y"]).unwrap_or(0),
                        url,
                        link: link.to_string(),
                    });
                }
            }
        }

        Ok(list)
    }
}
