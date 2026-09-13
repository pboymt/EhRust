//! 收藏夹页面的分类槽位信息。
//!
//! `favorites.php` 页面顶部以 10 个 `.fp` 槽位展示各收藏夹的名称与画廊数
//! （第 11 个 `.fp.fps` 是"全部收藏"汇总，不在本结构内），
//! 排序方式来自分页区的 `<select>` 当前选中项。
//!
//! # 示例
//!
//! ```rust,no_run
//! # use libeh::dto::favorites::FavoriteCategories;
//! # use scraper::{Element as _, Html};
//! # fn demo(html: &str) -> Result<(), libeh::error::Error> {
//! let d = Html::parse_document(html);
//! let favs = FavoriteCategories::parse(&d)?;
//! assert_eq!(favs.names.len(), 10);
//! # Ok(())
//! # }
//! ```

use scraper::Html;
use serde::{Deserialize, Serialize};

use crate::dto::search_result::SearchResult;
use crate::error::Error;
use crate::utils::scraper::{parse_to, selector, text_content};

/// 收藏夹分类槽位信息（favorites.php 顶部）。
///
/// [`FavoriteCategories::names`](crate::dto::favorites::FavoriteCategories::names) 与 [`FavoriteCategories::counts`](crate::dto::favorites::FavoriteCategories::counts)
/// 按下标 0–9 对应收藏夹 0–9。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FavoriteCategories {
    /// 10 个收藏夹的名称（页面渲染文本）。
    pub names: Vec<String>,
    /// 10 个收藏夹各自的画廊数量。
    pub counts: Vec<i64>,
    /// 当前排序方式（`fs` 参数原值，如 `p`=按发布时间 / `f`=按收藏时间）。
    pub fav_order: Option<String>,
}

impl FavoriteCategories {
    /// 从收藏夹页面解析分类槽位。
    ///
    /// # Errors
    ///
    /// 页面缺少 `.fp` 槽位结构时返回 [`Error::Parse`]（常见于未登录：
    /// 站点返回 `This page requires you to log on.` 页面）。
    pub fn parse(d: &Html) -> Result<Self, Error> {
        let mut result = FavoriteCategories::default();

        let fps = d.select(&selector(".fp")?).collect::<Vec<_>>();
        // 10 个分类槽位 + 1 个"全部"汇总槽位（.fp.fps）
        if fps.len() < 10 {
            return Err(Error::parse_msg(
                "favorite categories",
                "page does not contain 10 favorite slots (not logged in?)",
                String::new(),
            ));
        }
        for fp in fps.iter().take(10) {
            let cells: Vec<_> = fp.children().filter(|c| c.value().is_element()).collect();
            if cells.len() < 3 {
                continue;
            }
            // 计数与名称必须成对写入，避免个别槽位缺列时两个数组错位
            let count = scraper::ElementRef::wrap(cells[0])
                .map(|cell| parse_to::<i64>(&text_content(cell.text())).unwrap_or(0));
            let name = scraper::ElementRef::wrap(cells[2]).map(|cell| text_content(cell.text()));
            if let (Some(count), Some(name)) = (count, name) {
                result.counts.push(count);
                result.names.push(name);
            }
        }

        // 当前排序：searchnav 里 <select> 的选中项
        if let Some(selected) = d
            .select(&selector(".searchnav select option[selected]")?)
            .next()
        {
            result.fav_order = selected
                .value()
                .attr("value")
                .map(std::string::ToString::to_string);
        }

        Ok(result)
    }
}

/// 收藏夹页面的完整解析结果。
///
/// `favorites.php` 同时承载槽位统计（[`FavoriteCategories`]）与
/// 画廊列表（[`SearchResult`]），本结构把两者合并返回。
///
/// 获取与增删的调用方式见
/// [`EhClient::favorites`](crate::client::client::EhClient::favorites) /
/// [`EhClient::add_favorite`](crate::client::client::EhClient::add_favorite) /
/// [`EhClient::modify_favorites`](crate::client::client::EhClient::modify_favorites)。
#[derive(Debug, Clone, Default)]
pub struct FavoritesPage {
    /// 10 个收藏夹的名称与计数。
    pub categories: FavoriteCategories,
    /// 当前收藏夹内的画廊列表（含分页信息）。
    pub result: SearchResult,
}
