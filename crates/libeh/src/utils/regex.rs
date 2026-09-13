//! 正则辅助：带错误处理的编译与"忽略空白差异"的短语匹配。
use regex::Regex;

/// 编译正则表达式。
///
/// # Errors
///
/// 模式非法时返回 [`Error::Parse`](crate::error::Error::Parse)。
/// 对编译期已知的常量模式，建议配合 `LazyLock` 缓存（见 [`crate::dto::api`]）。
pub fn regex(regex: &str) -> Result<Regex, crate::error::Error> {
    Regex::new(regex).map_err(|err| crate::error::Error::parse("regex", err.to_string(), regex))
}

/// 判断 HTML 中是否包含某短语，忽略词间空白差异（换行/缩进）。
///
/// 站点页面常在文案内部换行——例如 "No hits found" 可能被渲染为
/// "No" 与换行缩进后再接 "hits found"——直接子串匹配不可靠；
/// 本函数把短语按词拆分、以 `\s+` 连接为正则后再匹配。
pub(crate) fn contains_phrase(haystack: &str, phrase: &str) -> bool {
    let pattern = phrase
        .split_whitespace()
        .map(::regex::escape)
        .collect::<Vec<_>>()
        .join(r"\s+");
    match Regex::new(&pattern) {
        Ok(re) => re.is_match(haystack),
        Err(_) => haystack.contains(phrase),
    }
}

#[cfg(test)]
mod tests {
    use super::contains_phrase;

    #[test]
    fn matches_across_whitespace() {
        let html = "No\n                    hits found</p>";
        assert!(contains_phrase(html, "No hits found"));
        assert!(!contains_phrase(html, "No results"));
    }
}
