//! 内部工具（不保证 SemVer 稳定）。
//!
//! - [`regex`]：带错误处理的正则编译；
//! - [`scraper`]：CSS 选择器、文本提取、站点行内样式（评分/收藏槽位）与日期解析；
//! - [`serde`]：api.php 数字字段的字符串形式反序列化。
pub mod regex;
pub mod scraper;
pub mod serde;

/// 反转义站点文本中的 XML/HTML 实体。
///
/// 覆盖站点实际会出现的五种实体：`&amp;`、`&lt;`、`&gt;`、`&quot;`、
/// `&#039;`（含数字形式 `&#39;`）。未识别的实体原样保留。
#[must_use]
pub fn unescape_xml(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#039;", "'")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod unescape_tests {
    #[test]
    fn unescapes_common_entities() {
        use super::unescape_xml;
        assert_eq!(
            unescape_xml("a&amp;b&#039;c&quot;d&lt;e&gt;f"),
            "a&b'c\"d<e>f"
        );
        // &amp; 必须最后处理，避免 &amp;lt; 被二次反转义
        assert_eq!(unescape_xml("&amp;lt;"), "&lt;");
    }
}
