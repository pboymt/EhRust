//! 列表页（`table.ptt` 数字分页）解析回归测试。
//!
//! 夹具迁移自 EhViewer `GalleryListParserTest` 的测试资源，
//! 覆盖 E 站/里站 × Minimal/MinimalPlus/Compact/Extended/Thumbnail
//! 五种列表版式；断言对齐 EhViewer 的测试意图：
//! 每个夹具解析出 25 行，且关键字段非空。

mod common;

use libeh::dto::search_result::SearchResult;

/// 10 个夹具：E/Ex 两站 × 5 种版式。
const FIXTURES: [&str; 10] = [
    "GalleryListParserTestEMinimal.html",
    "GalleryListParserTestEMinimalPlus.html",
    "GalleryListParserTestECompat.html",
    "GalleryListParserTestEExtended.html",
    "GalleryListParserTestEThumbnail.html",
    "GalleryListParserTestExMinimal.html",
    "GalleryListParserTestExMinimalPlus.html",
    "GalleryListParserTestExCompat.html",
    "GalleryListParserTestExExtended.html",
    "GalleryListParserTestExThumbnail.html",
];

#[test]
fn parses_25_rows_per_fixture() {
    for name in FIXTURES {
        let html = common::fixture(name);
        let result =
            SearchResult::parse(html).unwrap_or_else(|e| panic!("{name}: parse failed: {e}"));
        assert_eq!(result.gallery_info_list.len(), 25, "{name}: row count");
        for gi in &result.gallery_info_list {
            assert!(gi.gid > 0, "{name}: gid");
            assert_eq!(gi.token.len(), 10, "{name}: token");
            assert!(!gi.title.is_empty(), "{name}: title");
            assert!(!gi.thumb.is_empty(), "{name}: thumb");
            assert!(gi.posted_raw.is_some(), "{name}: posted");
        }
    }
}

#[test]
fn ptt_pages_and_next_page() {
    let html = common::fixture("GalleryListParserTestECompat.html");
    let result = SearchResult::parse(html).unwrap();
    // 该夹具的列表共 4 页，下一页链接为 page=1
    assert!(result.pages > 0, "pages should be parsed from ptt");
    assert!(result.next_page > 0, "next_page should be parsed from ptt");
}

#[test]
fn uploader_present_except_thumbnail_layout() {
    for name in FIXTURES {
        let html = common::fixture(name);
        let result = SearchResult::parse(html).unwrap();
        let with_uploader = result
            .gallery_info_list
            .iter()
            .filter(|gi| gi.uploader.is_some())
            .count();
        if name.contains("Thumbnail") {
            assert_eq!(with_uploader, 0, "{name}: thumbnail layout has no uploader");
        } else {
            assert!(with_uploader > 0, "{name}: uploader should be parsed");
        }
    }
}

#[test]
fn minimal_layout_rows_are_complete() {
    // 回归：老版解析器只在 glhide/gl3e 中找上传者/页数，对 Minimal 版式整行失败
    let html = common::fixture("GalleryListParserTestEMinimal.html");
    let result = SearchResult::parse(html).unwrap();
    assert!(result.skipped_rows == 0, "no rows should be skipped");
    let with_pages = result
        .gallery_info_list
        .iter()
        .filter(|gi| gi.pages > 0)
        .count();
    assert!(with_pages > 0, "pages should be parsed in minimal layout");
}

#[test]
fn rating_parsed_in_range() {
    let html = common::fixture("GalleryListParserTestECompat.html");
    let result = SearchResult::parse(html).unwrap();
    let rated = result
        .gallery_info_list
        .iter()
        .filter(|gi| gi.rating > 0.0)
        .count();
    assert!(rated > 0, "ratings should be parsed");
}
