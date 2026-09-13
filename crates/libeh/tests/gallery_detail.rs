//! 画廊详情页解析回归测试。
//!
//! 夹具 `GalleryDetail.html` 迁移自 EhViewer 测试资源（`apiuid` 已脱敏为
//! `1000000`）。验证：核心会话参数、元数据键值表、多冒号标签（P1-1 回归）、
//! 预览接线（P2-1）、评论投票对账。

mod common;

use libeh::dto::gallery::detail::GalleryDetail;

fn parse_fixture() -> GalleryDetail {
    let html = common::fixture("GalleryDetail.html");
    GalleryDetail::parse(html).unwrap_or_else(|e| panic!("detail parse failed: {e}"))
}

#[test]
fn parses_core_params() {
    let detail = parse_fixture();
    assert_eq!(detail.info.gid, 3101249);
    assert_eq!(detail.info.token, "7fb0ea5a0e");
    assert_eq!(
        detail.api_uid, 1_000_000,
        "apiuid should equal the sanitized value"
    );
    assert!(!detail.api_key.is_empty());
}

#[test]
fn parses_metadata() {
    let detail = parse_fixture();
    assert!(!detail.info.title.is_empty());
    assert!(matches!(
        detail.info.category,
        libeh::dto::gallery::category::Category::GameCG
    ));
    assert!(detail.info.pages > 0, "Length row should be parsed");
    assert!(detail.rating_count > 0);
    assert!(detail.info.rating > 0.0);
    assert!(
        detail.info.posted_raw.is_some(),
        "posted_raw should be filled"
    );
    assert!(!detail.language.is_empty());
    assert!(!detail.size.is_empty());
}

#[test]
fn torrent_and_archive_links_are_text_anchored() {
    let detail = parse_fixture();
    assert_eq!(detail.torrent_count, 5);
    assert!(
        detail.torrent_url.contains("gallerytorrents.php"),
        "{}",
        detail.torrent_url
    );
    assert!(
        detail.archive_url.contains("archiver.php"),
        "{}",
        detail.archive_url
    );
}

#[test]
fn multi_colon_tags_survive() {
    // 回归 P1-1：值含冒号的标签（如 parody:re:zero）不得让整页解析失败
    let detail = parse_fixture();
    assert!(!detail.info.tags.is_empty());
    for tag in &detail.info.tags {
        assert!(
            tag.namespace().is_some() || matches!(tag, libeh::dto::keyword::Keyword::Normal(_))
        );
    }
}

#[test]
fn preview_is_wired() {
    // 回归 P2-1：preview 字段必须由 GalleryPreview::parse 填充
    let detail = parse_fixture();
    assert_eq!(detail.preview.total, 838, "total images");
    assert!(detail.preview.total_set > 0, "preview pages");
    assert_eq!(detail.preview.pages.len(), 20, "first preview page entries");
    let first = &detail.preview.pages[0];
    assert_eq!(first.page, 1);
    assert!(first.link.contains("/s/"), "page link");
    assert!(first.url.starts_with("https://"), "thumb url");
}

#[test]
fn comments_vote_accounting() {
    let detail = parse_fixture();
    assert!(!detail.comments.is_empty());
    for comment in &detail.comments {
        if comment.vote_state.more == 0 {
            let sum = comment.vote_state.base
                + comment
                    .vote_state
                    .votes
                    .iter()
                    .map(|v| v.score)
                    .sum::<i64>();
            assert_eq!(
                sum, comment.score,
                "vote mismatch on comment {:?}",
                comment.id
            );
        }
    }
}
