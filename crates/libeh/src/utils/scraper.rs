//! scraper 的辅助函数：CSS 选择器、文本提取与站点特有样式/日期的解析。
//!
//! 本模块是各 HTML 解析器（[`crate::dto`]）的公共工具层：
//!
//! - [`selector`] / [`text_content`]：DOM 查询与文本归一化；
//! - [`parse_posted`]：站点页面渲染的时间字符串 → [`DateTime<Utc>`]；
//! - [`parse_rating`]：星级图标行内样式的像素偏移 → 评分数值；
//! - [`parse_favorite_slot`]：收藏槽位的背景色 → 槽位编号（0–9）；
//! - [`deepest_text`]：取元素内最深叶子节点的文本（列表标题）。
//!
//! # 站点样式编码
//!
//! 站点把部分数值"藏"在行内样式里，解析规则与 EhViewer 保持一致：
//!
//! - 评分图标 `background-position: -64px -21px`：x 偏移每 16px 减 1 星，
//!   y 偏移 `21px` 表示半星（`5 - |x|/16 - 0.5`）；
//! - 收藏槽位 `background-color:rgba(r,g,b,…)`：10 组固定 RGB
//!   （黑/红/橙/黄/绿/浅绿/蓝/深蓝/紫/粉）映射到收藏夹 0–9。

use std::str::FromStr;

use chrono::{DateTime, NaiveDateTime, Utc};
use scraper::{element_ref::Text, ElementRef, Selector};

use super::regex::regex;

/// 根据给定的 CSS 选择器字符串创建一个选择器对象。
///
/// # Errors
///
/// 选择器语法非法时返回 [`Error::Parse`](crate::error::Error::Parse)。
pub fn selector(selector: &str) -> Result<Selector, crate::error::Error> {
    Selector::parse(selector)
        .map_err(|e| crate::error::Error::parse("css selector", e.to_string(), selector))
}

/// 把元素内的全部文本片段归一化为单个字符串。
///
/// 各片段内的**连续空白折叠为单个空格**：站点 HTML 常在锚点文本内
/// 跨行换行（例如 "Torrent\n    Download (5)" 会被归一化为
/// "Torrent Download (5)"），片段间以空格连接，再去除首尾空白。
pub fn text_content(text: Text) -> String {
    let seq: Vec<String> = text
        .map(|t| t.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|s| !s.is_empty())
        .collect();
    seq.join(" ").trim().to_string()
}

/// 取元素内**最深叶子节点**的文本。
///
/// 站点列表的标题元素（`.glname`）内层结构随版式变化，
/// 但标题文本总在最深的叶子节点上（EhViewer 同款策略）。
pub fn deepest_text(element: ElementRef) -> String {
    let mut current = element;
    while let Some(next) = current
        .children()
        .find(|c| c.value().is_element())
        .and_then(ElementRef::wrap)
    {
        current = next;
    }
    text_content(current.text())
}

/// 解析站点页面渲染的时间字符串（`%Y-%m-%d %H:%M`）。
///
/// # 注意（时区）
///
/// 页面按查看者账号的时区渲染时间，而本函数**按 UTC 解释**；
/// 与 API（`gdata` 返回 unix 时间戳，真 UTC）对比时可能偏差数小时。
/// 需要精确时间请使用 API 路径（[`crate::dto::api::GalleryMetadata::posted`]）。
///
/// # Errors
///
/// 格式不匹配时返回 [`Error::Parse`](crate::error::Error::Parse)。
pub fn parse_posted(text: &str) -> Result<DateTime<Utc>, crate::error::Error> {
    NaiveDateTime::parse_from_str(text.trim(), "%Y-%m-%d %H:%M")
        .map(|naive| naive.and_utc())
        .map_err(|e| crate::error::Error::parse("posted datetime", e.to_string(), text))
}

/// 收藏夹槽位颜色（与站点 `favorites.php` 的行内样式一致）。
struct FavoriteSlotRgba(i16, i16, i16);

/// 从行内样式中提取 `background-color:rgba(r,g,b,…)` 的 RGB 分量。
fn parse_style_color(text: &str) -> Result<FavoriteSlotRgba, crate::error::Error> {
    let r = regex(r"background-color:rgba\((\d+),(\d+),(\d+),")?;
    match r.captures(text) {
        Some(caps) => {
            let (r, g, b) = (
                caps[1].parse::<i16>(),
                caps[2].parse::<i16>(),
                caps[3].parse::<i16>(),
            );
            match (r, g, b) {
                (Ok(r), Ok(g), Ok(b)) => Ok(FavoriteSlotRgba(r, g, b)),
                _ => Err(crate::error::Error::parse_msg(
                    "favorite slot color",
                    "rgb components out of range",
                    text,
                )),
            }
        }
        None => Err(crate::error::Error::parse_msg(
            "favorite slot color",
            "no rgba() in style",
            text,
        )),
    }
}

/// 解析收藏夹槽位：`posted_` 单元格的背景色 → 收藏夹编号（0–9）。
///
/// # Errors
///
/// 样式中没有可识别的槽位颜色时返回错误
/// （未收藏的画廊通常没有该样式，调用方应把错误视为"未知/未收藏"）。
pub fn parse_favorite_slot(text: &str) -> Result<isize, crate::error::Error> {
    let rgb = parse_style_color(text)?;
    let slot = match rgb {
        FavoriteSlotRgba(0, 0, 0) => 0,
        FavoriteSlotRgba(240, 0, 0) => 1,
        FavoriteSlotRgba(240, 160, 0) => 2,
        FavoriteSlotRgba(208, 208, 0) => 3,
        FavoriteSlotRgba(0, 128, 0) => 4,
        FavoriteSlotRgba(144, 240, 64) => 5,
        FavoriteSlotRgba(64, 176, 240) => 6,
        FavoriteSlotRgba(0, 0, 240) => 7,
        FavoriteSlotRgba(80, 0, 128) => 8,
        FavoriteSlotRgba(224, 128, 224) => 9,
        _ => {
            return Err(crate::error::Error::parse_msg(
                "favorite slot",
                "unknown slot color",
                text,
            ))
        }
    };
    Ok(slot)
}

/// 解析评分图标的行内样式为评分数值（0.0–5.0，含 0.5 半星）。
///
/// 图标以雪碧图实现，`background-position` 的像素偏移反向映射星级：
/// `x` 偏移每 16px 减 1 星，`y=21px` 表示半星。
///
/// # Errors
///
/// 样式中没有两个像素偏移值时返回 [`Error::Parse`](crate::error::Error::Parse)。
pub fn parse_rating(text: &str) -> Result<f32, crate::error::Error> {
    let r = regex(r"-?(\d+)px -?(\d+)px")?;
    let Some(caps) = r.captures(text) else {
        return Err(crate::error::Error::parse_msg(
            "rating style",
            "no pixel offsets",
            text,
        ));
    };
    let Ok(major) = caps[1].parse::<i32>() else {
        return Err(crate::error::Error::parse_msg(
            "rating style",
            "bad x offset",
            text,
        ));
    };
    let Ok(patch) = caps[2].parse::<i32>() else {
        return Err(crate::error::Error::parse_msg(
            "rating style",
            "bad y offset",
            text,
        ));
    };
    let rating = 5.0 - major as f32 / 16.0;
    Ok(if patch == 21 { rating - 0.5 } else { rating })
}

/// 转换字符串为指定类型（类型需实现 [`FromStr`](std::str::FromStr)）。
///
/// # Errors
///
/// 解析失败时返回 [`Error::Parse`](crate::error::Error::Parse)。
pub fn parse_to<T: FromStr>(value: &str) -> Result<T, crate::error::Error> {
    value
        .parse::<T>()
        .map_err(|_| crate::error::Error::parse_msg("number", "parse error", value))
}
