//! 搜索页（`div.searchnav` 快速分页）与收藏夹解析回归测试。
//!
//! 夹具迁移自 EhViewer 测试资源：
//! `FavoritesListParser.html`（searchnav + 收藏夹槽位）、
//! `EmptyGalleryList.html`（无结果页）。

mod common;

use libeh::dto::favorites::FavoriteCategories;
use libeh::dto::search_result::SearchResult;

#[test]
fn searchnav_hrefs_are_extracted() {
    let html = common::fixture("FavoritesListParser.html");
    let result = SearchResult::parse(html).unwrap();
    // 夹具是收藏夹第 1 页：没有 first/prev 链接（渲染为无 href 的 span），只有 next/last
    assert!(result.first_href.is_none(), "page 1 has no ufirst link");
    assert!(result.prev_href.is_none(), "page 1 has no uprev link");
    assert!(result.next_href.is_some(), "unext");
    assert!(result.last_href.is_some(), "ulast");
    assert!(
        !result.gallery_info_list.is_empty(),
        "rows should be parsed"
    );
}

#[test]
fn favorite_categories_parsed() {
    let html = common::fixture("FavoritesListParser.html");
    let d = scraper::Html::parse_document(&html);
    let favs = FavoriteCategories::parse(&d).unwrap();
    assert_eq!(favs.names.len(), 10);
    assert_eq!(favs.counts.len(), 10);
    assert!(
        favs.counts.iter().all(|c| *c >= 0),
        "counts should be non-negative"
    );
}

#[test]
fn empty_result_view_has_no_rows() {
    // 夹具为"过滤过严/页码越界"的空视图（文案 No unfiltered results in this page range）：
    // 表格只剩表头，不应解析出任何行
    let html = common::fixture("EmptyGalleryList.html");
    let result = SearchResult::parse(html).unwrap();
    assert!(result.gallery_info_list.is_empty());
    assert_eq!(result.skipped_rows, 0);
}
