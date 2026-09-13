//! 图片页（`/s/{pToken}/{gid}-{page}`）的解析器。
//!
//! 输入为单张图片页的 HTML，输出 [`GalleryPage`](crate::dto::gallery::page::GalleryPage)——"能看图"所需的全套参数：
//!
//! - **图片地址**：`<img id="img" src="…">`，通常是 H@H 客户端直链
//!   （带 `keystamp` 时效签名），下载时需携带图片页的 `Referer`；
//! - **`showkey`**：内联脚本中的会话参数，调用 api.php `showpage`
//!   方法翻页时必需（见 [`crate::client::client::EhClient::gallery_page`]）；
//! - **`skipHathKey`**：`nl('…')` 调用的参数——当前 H@H 源失联时，
//!   把 `?nl={skip_hath_key}` 追加到图片 URL 重新请求即可换源；
//! - **原图链接**（两种形态，可能同时存在或都不存在）：
//!   - [`GalleryPage::origin_image_url`](crate::dto::gallery::page::GalleryPage::origin_image_url)：`fullimg.php?…` 链接，
//!     需要账户有原片权限（GP/星星），GET 它会 302 到真实原图；
//!   - [`GalleryPage::original_direct_url`](crate::dto::gallery::page::GalleryPage::original_direct_url)：新版页面用
//!     `prompt('Copy the URL below.', '…')` 直接给出的原图地址。
//!
//! # 示例
//!
//! ```rust,no_run
//! # use libeh::dto::gallery::page::GalleryPage;
//! # fn demo(html: String) -> Result<(), libeh::error::Error> {
//! let page = GalleryPage::parse(html)?;
//! println!("image: {}", page.image_url);
//! if let Some(origin) = page.origin_image_url {
//!     println!("original (needs permission): {origin}");
//! }
//! # Ok(())
//! # }
//! ```

use std::sync::LazyLock;

use regex::Regex;

use crate::error::Error;

/// 图片地址：`<img id="img" src="…">`。
static PATTERN_IMAGE_URL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<img[^>]*id="img"[^>]*src="([^"]+)"[^>]*>"#).expect("constant regex")
});
/// 翻页会话参数：`var showkey="…";`。
static PATTERN_SHOW_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"var showkey="([0-9a-z]+)";"#).expect("constant regex"));
/// 换源参数：`onclick="return nl('…')"`（`#loadfail` 链接）。
static PATTERN_SKIP_HATH_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"return nl\('([^)]+)'\)").expect("constant regex"));
/// 原图页链接（旧式）：`<a href="…fullimg…">`。
static PATTERN_ORIGIN_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<a href="([^"]*fullimg[^"]*)""#).expect("constant regex"));
/// 原图直链（新式）：`prompt('Copy the URL below.', '…')`。
static PATTERN_ORIGIN_PROMPT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"prompt\('Copy the URL below\.', '([^']+)'\)").expect("constant regex")
});

/// 单张图片页的解析结果。
///
/// 各字段的站点语义见[模块文档](self)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalleryPage {
    /// 当前页图片地址（H@H 直链或站点直链，带时效签名）。
    pub image_url: String,
    /// 翻页会话参数（api.php `showpage` 方法的 `showkey` 入参）。
    pub show_key: String,
    /// 换源参数：图片 URL 追加 `?nl={skip_hath_key}` 可请求更换 H@H 源。
    pub skip_hath_key: Option<String>,
    /// 原图页链接（`fullimg.php?…`，GET 后 302 到原图；需要原片权限）。
    pub origin_image_url: Option<String>,
    /// 原图直链（新版页面 `prompt()` 给出的地址，无需再跳转）。
    pub original_direct_url: Option<String>,
}

impl GalleryPage {
    /// 解析图片页 HTML。
    ///
    /// # Errors
    ///
    /// 缺少图片地址或 `showkey`（页面结构变化、被重定向到错误页等）时
    /// 返回 [`Error::Parse`]，`snippet` 携带原始 HTML 片段。
    pub fn parse(html: &str) -> Result<Self, Error> {
        let image_url = PATTERN_IMAGE_URL
            .captures(html)
            .map(|c| crate::utils::unescape_xml(c[1].trim()))
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                Error::parse(
                    "gallery page",
                    "no <img id=\"img\"> element",
                    crate::error::snippet(html, 512),
                )
            })?;
        let show_key = PATTERN_SHOW_KEY
            .captures(html)
            .map(|c| c[1].to_string())
            .ok_or_else(|| {
                Error::parse(
                    "gallery page",
                    "no showkey in inline script",
                    crate::error::snippet(html, 512),
                )
            })?;

        Ok(GalleryPage {
            image_url,
            show_key,
            skip_hath_key: PATTERN_SKIP_HATH_KEY
                .captures(html)
                .map(|c| crate::utils::unescape_xml(c[1].trim())),
            origin_image_url: PATTERN_ORIGIN_LINK
                .captures(html)
                .map(|c| crate::utils::unescape_xml(c[1].trim())),
            original_direct_url: PATTERN_ORIGIN_PROMPT
                .captures(html)
                .map(|c| crate::utils::unescape_xml(c[1].trim())),
        })
    }

    /// 生成"换源后"的图片 URL：在原地址上追加 `?nl={skip_hath_key}`。
    ///
    /// 当前 H@H 源超时/失联时调用；返回 `None` 表示页面没有提供
    /// 换源参数。地址已含 query 时使用 `&` 连接。
    #[must_use]
    pub fn image_url_with_skip_hath(&self) -> Option<String> {
        let key = self.skip_hath_key.as_ref()?;
        let sep = if self.image_url.contains('?') {
            '&'
        } else {
            '?'
        };
        Some(format!("{}{}nl={}", self.image_url, sep, key))
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    use super::GalleryPage;

    fn load_fixture(name: &str) -> String {
        let path = format!(
            "{}/tests/fixtures/pending/{name}",
            env!("CARGO_MANIFEST_DIR")
        );
        let mut file = File::open(&path).unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        buf
    }

    #[test]
    fn parses_page_fixture() {
        let html = load_fixture("GalleryPageParserTest.html");
        let page = GalleryPage::parse(&html).unwrap();
        assert!(page.image_url.contains("/h/"), "{}", page.image_url);
        assert_eq!(page.show_key, "ghz0e5m98a4");
        assert_eq!(page.skip_hath_key.as_deref(), Some("26664-430636"));
        assert!(
            page.origin_image_url
                .as_deref()
                .is_some_and(|u| u.contains("fullimg.php?gid=1363978")),
            "{:?}",
            page.origin_image_url
        );
        assert!(
            page.original_direct_url
                .as_deref()
                .is_some_and(|u| u.contains("/r/")),
            "{:?}",
            page.original_direct_url
        );
        // 换源 URL：分隔符取决于原地址是否已含 query（夹具地址无 query，用 ?）
        let skip = page.image_url_with_skip_hath().unwrap();
        assert!(skip.contains("?nl=26664-430636"), "{skip}");
    }

    #[test]
    fn js_loaded_page_variant_is_a_clear_parse_error() {
        // newPage.html 是"JS 加载图片"的过渡页变体（图片由脚本插入，
        // 无 <img id="img"> 与 showkey）——纯 HTML 解析器对其给出明确错误，
        // 调用方应回退到 api.php showpage 方法（EhClient::gallery_page）
        let html = load_fixture("newPage.html");
        let err = GalleryPage::parse(&html).unwrap_err();
        assert!(matches!(
            err,
            crate::error::Error::Parse {
                context: "gallery page",
                ..
            }
        ));
    }

    #[test]
    fn broken_page_is_parse_error() {
        let err = GalleryPage::parse("<html><body>hello</body></html>").unwrap_err();
        assert!(matches!(err, crate::error::Error::Parse { .. }));
    }
}
