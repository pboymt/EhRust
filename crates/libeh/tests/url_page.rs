//! 页面 URL 解析（`PageListItem`）回归测试。
//!
//! 用例迁移自 EhViewer `GalleryPageUrlParserTest` 的参数化表。

use libeh::dto::api::PageListItem;

#[test]
fn strict_and_loose_cases_from_ehviewer() {
    // (输入, 是否期望成功, 期望 gid/ptoken/page)
    type Case = (&'static str, bool, Option<(i64, &'static str, i32)>);
    let cases: &[Case] = &[
        (
            "https://e-hentai.org/s/7b87643838/530350-1",
            true,
            Some((530350, "7b87643838", 1)),
        ),
        (
            "https://exhentai.org/s/7b87643838/530350-1",
            true,
            Some((530350, "7b87643838", 1)),
        ),
        ("7b87643838/530350-1", false, None),
        ("530350/8b3c7e4a21", false, None),
        ("https://e-hentai.org/g/530350/8b3c7e4a21/", false, None),
    ];
    for (input, ok, expected) in cases {
        let result = PageListItem::try_from((*input).to_string());
        assert_eq!(result.is_ok(), *ok, "{input:?}");
        if let (true, Some((gid, ptoken, page))) = (result.is_ok(), *expected) {
            let item = result.unwrap();
            assert_eq!(item.0, gid, "{input:?}");
            assert_eq!(item.1, ptoken, "{input:?}");
            assert_eq!(item.2, page, "{input:?}");
        }
    }
}
