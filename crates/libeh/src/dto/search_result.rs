//! 搜索结果页的解析器。
//!
//! 输入为列表页/搜索页的 HTML，输出 [`SearchResult`](crate::dto::search_result::SearchResult)。
//! 与 EhViewer 的 `GalleryListParser` 对齐，覆盖：
//!
//! - **两种分页结构**：搜索页的 `div.searchnav`（`ufirst/uprev/unext/ulast`
//!   四个跳转锚点，即"快速分页"）与列表页的 `table.ptt`（数字总页数 +
//!   下一页链接）；
//! - **五种列表版式**：Minimal / MinimalPlus / Compact（兼容）/
//!   Extended / Thumbnail——各版式的标题、缩略图、页数、上传者位置不同，
//!   解析器按 `glthumb → gl1e → gl3t`（缩略图）等顺序回退；
//! - **结果计数**：`searchtext` 中的 `Found X results` 文本（含
//!   `thousands`/`about` 模糊量的归一化）；
//! - **空结果与订阅提示**：`No hits found` 与
//!   `You do not have any watched tags`。
//!
//! # 行解析的容错
//!
//! 单行解析失败不会中断整体：该行被跳过并计入
//! [`SearchResult::skipped_rows`](crate::dto::search_result::SearchResult::skipped_rows)（`log::warn!` 记录原因），
//! 头部行（无 `.glname`）不计入跳过数。
//!
//! # 示例
//!
//! ```rust,no_run
//! # use libeh::dto::search_result::SearchResult;
//! # fn demo(html: String) -> Result<(), libeh::error::Error> {
//! let result = SearchResult::parse(html)?;
//! println!("{} galleries, total pages {}", result.gallery_info_list.len(), result.pages);
//! # Ok(())
//! # }
//! ```

use crate::error::Error;
use crate::url::gallery::GalleryBuilder;
use crate::utils::scraper::{
    deepest_text, parse_favorite_slot, parse_posted, parse_rating, parse_to, selector, text_content,
};
use scraper::{ElementRef, Html};
use serde::{Deserialize, Serialize};

use super::gallery::category::Category;
use super::gallery::info::GalleryInfo;
use super::keyword::Keyword;

/// 搜索结果。
///
/// 字段含义与填充规则见[模块文档](self)；未从页面取得的信息保持
/// [`SearchResult::default`](crate::dto::search_result::SearchResult::default) 的默认值。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// 列表总页数（`table.ptt` 分支填充；searchnav 分支无此信息，为 `-1`）。
    pub pages: isize,
    /// 下一页的数字页码（`ptt` 分支；无下一页或非 ptt 分支为 `-1`）。
    pub next_page: isize,
    /// 结果计数文本（如 `"Found 1,234 results"` 归一化后的数量部分）。
    pub result_count: Option<String>,
    /// 首页锚点（searchnav）。
    pub first_href: Option<String>,
    /// 上一页锚点（searchnav）。
    pub prev_href: Option<String>,
    /// 下一页锚点（searchnav）。
    pub next_href: Option<String>,
    /// 末页锚点（searchnav）。
    pub last_href: Option<String>,
    /// 订阅页无任何监视标签（页面原文 `You do not have any watched tags`）。
    pub no_watched_tags: bool,
    /// 因结构异常被跳过的行数（头部行不计入；正常应为 0）。
    pub skipped_rows: usize,
    /// 画廊条目列表。
    pub gallery_info_list: Vec<GalleryInfo>,
}

impl Default for SearchResult {
    fn default() -> Self {
        SearchResult {
            pages: -1,
            next_page: -1,
            result_count: None,
            first_href: None,
            prev_href: None,
            next_href: None,
            last_href: None,
            no_watched_tags: false,
            skipped_rows: 0,
            gallery_info_list: vec![],
        }
    }
}

impl SearchResult {
    /// 解析列表/搜索页 HTML。
    ///
    /// # Errors
    ///
    /// 页面既无 `searchnav` 也无 `ptt` 分页结构（排除"无结果"页面）时
    /// 返回 [`Error::Parse`]，`snippet` 携带原始 HTML 片段。
    pub fn parse(html: String) -> Result<Self, Error> {
        let mut result = SearchResult::default();

        // 订阅页无监视标签的提示（不视为解析失败）
        if crate::utils::regex::contains_phrase(&html, "You do not have any watched tags") {
            result.no_watched_tags = true;
        }

        // 空结果分支：站点明确说明没有命中
        if crate::utils::regex::contains_phrase(&html, "No hits found") {
            result.pages = 0;
            return Ok(result);
        }

        // 未登录访问收藏夹等页面：站点返回登录引导页而非列表
        if crate::utils::regex::contains_phrase(&html, "This page requires you to log on.") {
            return Err(Error::Protocol("This page requires you to log on.".into()));
        }

        let d = Html::parse_document(&html);

        Self::parse_search_nav(&d, &mut result)?;
        Self::parse_ptt(&d, &mut result);
        result.result_count = Self::parse_result_count(&d);
        Self::parse_rows(&d, &mut result);

        Ok(result)
    }

    /// 解析 `searchnav` 快速分页（ufirst/uprev/unext/ulast）。
    ///
    /// 页面缺少 searchnav（如列表页只带 ptt）时为空操作。
    fn parse_search_nav(d: &Html, result: &mut SearchResult) -> Result<(), Error> {
        let Some(search_nav) = d.select(&selector(".searchnav")?).next() else {
            return Ok(());
        };
        for (id, slot) in [
            ("#ufirst", &mut result.first_href),
            ("#uprev", &mut result.prev_href),
            ("#unext", &mut result.next_href),
            ("#ulast", &mut result.last_href),
        ] {
            if let Some(a) = search_nav.select(&selector(id)?).next() {
                *slot = a.value().attr("href").map(std::string::ToString::to_string);
            }
        }
        Ok(())
    }

    /// 解析 `table.ptt` 数字分页：总页数与下一页页码。
    ///
    /// 页面缺少 ptt（如 favorites 的快速分页）时为空操作。
    fn parse_ptt(d: &Html, result: &mut SearchResult) {
        let Ok(s) = selector("table.ptt") else { return };
        let Some(ptt) = d.select(&s).next() else {
            return;
        };
        // 行内单元格：倒数第二个是"总页数"，最后一个链接是"下一页"
        let Ok(tds) = selector("tr > td") else { return };
        let cells: Vec<ElementRef> = ptt.select(&tds).collect();
        if cells.len() >= 2 {
            if let Ok(pages) = parse_to::<isize>(&text_content(cells[cells.len() - 2].text())) {
                result.pages = pages;
            }
        }
        if let Some(last_link) = cells
            .last()
            .and_then(|td| td.select(&selector("a").ok()?).next())
        {
            if let Some(href) = last_link.value().attr("href") {
                if let Ok(r) = crate::utils::regex::regex(r"page=(\d+)") {
                    if let Some(caps) = r.captures(href) {
                        result.next_page = parse_to::<isize>(&caps[1]).unwrap_or(-1);
                    }
                }
            }
        }
    }

    /// 解析 `searchtext` 中的结果计数（`Found … results`）。
    ///
    /// 站点对大结果量使用模糊文案：`Found thousands of results` → `1,000+`、
    /// `Found about 1,234 results` → `1,234+`；无该元素时返回 `None`。
    fn parse_result_count(d: &Html) -> Option<String> {
        let text = text_content(d.select(&selector(".searchtext").ok()?).next()?.text());
        let r = crate::utils::regex::regex(r"Found .* results").ok()?;
        let matched = r.find(&text)?;
        let parts: Vec<&str> = matched.as_str().split(' ').collect();
        Some(match parts.len() {
            0..=2 => String::new(),
            3 => parts[1].to_string(),
            _ => match parts[1] {
                "thousands" => "1,000+".to_string(),
                "about" => format!("{}+", parts[2]),
                _ => parts[1..parts.len() - 1].concat(),
            },
        })
    }

    /// 收集并解析画廊行。
    ///
    /// 兼容表格版式（`table.itg tr`）与 div 版式（`div.itg > div`）；
    /// 无 `.glname` 的行视为表头行跳过。
    fn parse_rows(d: &Html, result: &mut SearchResult) {
        let rows: Vec<ElementRef> = match selector("table.itg tr") {
            Ok(s) => d.select(&s).collect(),
            Err(_) => return,
        };
        let rows = if rows.is_empty() {
            match selector("div.itg > div") {
                Ok(s) => d.select(&s).collect(),
                Err(_) => return,
            }
        } else {
            rows
        };
        let Ok(glname_sel) = selector(".glname") else {
            return;
        };
        for row in rows {
            // 表头行没有 glname，直接跳过且不计入 skipped_rows
            if row.select(&glname_sel).next().is_none() {
                continue;
            }
            match Self::parse_gallery_info(row) {
                Ok(gallery_info) => result.gallery_info_list.push(gallery_info),
                Err(err) => {
                    log::warn!("skipped gallery row: {err}");
                    result.skipped_rows += 1;
                }
            }
        }
    }

    /// 解析单个画廊行。
    ///
    /// # Errors
    ///
    /// 行内缺少标题链接、gid/token 无法解析或分类/页数缺失时返回
    /// [`Error::Parse`](crate::error::Error::Parse)（由调用方跳过该行）。
    fn parse_gallery_info(tr: ElementRef) -> Result<GalleryInfo, Error> {
        let mut gi = GalleryInfo::default();

        // 标题与 gid/token：全部版式都有 .glname，标题在 .glink 或最深叶子
        let glname = tr
            .select(&selector(".glname")?)
            .next()
            .ok_or_else(|| Error::parse_msg("gallery row", "no glname", String::new()))?;
        let anchor = glname
            .select(&selector("a")?)
            .next()
            .or_else(|| {
                // 老版式中 glname 的父节点就是链接
                let parent = glname.parent()?;
                let el = parent.value().as_element()?;
                if el.name() == "a" {
                    ElementRef::wrap(parent)
                } else {
                    None
                }
            })
            .ok_or_else(|| Error::parse_msg("gallery row", "no gallery link", String::new()))?;
        let href = anchor
            .value()
            .attr("href")
            .ok_or_else(|| Error::parse_msg("gallery row", "link has no href", String::new()))?;
        let builder = GalleryBuilder::parse(href.to_string())?;
        gi.gid = builder.gid;
        gi.token = builder.token;

        gi.title = match glname.select(&selector(".glink")?).next() {
            Some(glink) => text_content(glink.text()),
            None => deepest_text(glname),
        };
        if gi.title.is_empty() {
            return Err(Error::parse_msg(
                "gallery row",
                "title is empty",
                String::new(),
            ));
        }

        // 标签：.gt / .gtl 的 title 属性（命名空间:标签）
        for tag_el in tr.select(&selector(".gt, .gtl")?) {
            if let Some(tag) = tag_el.value().attr("title") {
                gi.tags.push(Keyword::from(tag.to_string()));
            }
        }

        // 分类：.cn（正常）/ .cs（压缩）
        let cn_sel = selector(".cn")?;
        let cs_sel = selector(".cs")?;
        let category_text = tr
            .select(&cn_sel)
            .next()
            .or_else(|| tr.select(&cs_sel).next())
            .map(|el| text_content(el.text()))
            .unwrap_or_default();
        gi.category = Category::from(category_text);

        // 缩略图：glthumb（常规）→ gl1e（extended）→ gl3t（thumbnail）
        let img = tr
            .select(&selector(".glthumb img")?)
            .next()
            .or_else(|| tr.select(&selector(".gl1e img").ok()?).next())
            .or_else(|| tr.select(&selector(".gl3t img").ok()?).next());
        if let Some(img) = img {
            gi.thumb = img
                .value()
                .attr("data-src")
                .or_else(|| img.value().attr("src"))
                .unwrap_or_default()
                .to_string();
        }

        // 页数：行内第一处 "N pages" 文本
        let row_text = text_content(tr.text());
        let pages_re = crate::utils::regex::regex(r"(?<page>\d+) pages?")?;
        if let Some(caps) = pages_re.captures(&row_text) {
            gi.pages = parse_to::<i64>(&caps["page"])?;
        }

        // 上传时间与收藏槽位：#posted_{gid} 单元格
        let posted_sel = selector(&format!("#posted_{}", gi.gid))?;
        if let Some(posted) = tr.select(&posted_sel).next() {
            let posted_text = text_content(posted.text());
            gi.posted = parse_posted(&posted_text)?;
            gi.posted_raw = Some(posted_text);
            if let Some(style) = posted.value().attr("style") {
                // 无法识别的槽位颜色视为"未知"（-2），与 EhViewer 语义一致
                gi.favorite_slot = parse_favorite_slot(style).unwrap_or(-2);
            }
        }

        // 评分：.ir 图标的行内样式
        if let Some(ir) = tr.select(&selector(".ir")?).next() {
            if let Some(style) = ir.value().attr("style") {
                gi.rating = parse_rating(style)?;
            }
        }

        // 上传者：指向 /uploader/ 的锚点；"(Disowned)" 视为无上传者
        for a in tr.select(&selector("a[href*='/uploader/']")?) {
            let name = text_content(a.text());
            if !name.is_empty() {
                if name != "(Disowned)" && name != "(Renamed)" {
                    gi.uploader = Some(name);
                }
                break;
            }
        }

        Ok(gi)
    }
}
