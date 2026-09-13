//! 论坛（IPB）登录响应的解析器。
//!
//! 账密登录向 `forums.e-hentai.org/index.php?act=Login&CODE=01` 提交表单，
//! 成功时响应含 `You are now logged in as: {昵称}`；失败时响应含
//! IPB 风格的错误框（`The error returned was:` 或 `.postcolor` 块）。
//! 解析规则对齐 EhViewer 的 `SignInParser`。
//!
//! # 示例
//!
//! ```rust
//! use libeh::dto::signin::parse_sign_in;
//!
//! let name = parse_sign_in("<p>You are now logged in as: Someone</p>").unwrap();
//! assert_eq!(name, "Someone");
//!
//! let err = parse_sign_in("<h4>The error returned was:</h4><p>bad password</p>").unwrap_err();
//! assert!(matches!(err, libeh::error::Error::Protocol(m) if m.contains("bad password")));
//! ```

use std::sync::LazyLock;

use regex::Regex;

use crate::error::Error;

/// 登录成功的欢迎语（捕获组为昵称）。
static NAME_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"You are now logged in as: (.+?)<").expect("constant regex"));
/// IPB 错误框：`The error returned was:` 后的错误说明，
/// 或 `.postcolor` 块内的错误文本。
static ERROR_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:<h4>The error returned was:</h4>\s*<p>(.+?)</p>)|(?:<span class="postcolor">(.+?)</span>)"#)
        .expect("constant regex")
});

/// 解析论坛登录响应。
///
/// # Returns
///
/// 登录成功时的账户昵称（页面渲染文本）。
///
/// # Errors
///
/// - IPB 错误框（密码错误、验证码要求等）→ [`Error::Protocol`]，
///   `msg` 为站点返回的错误原文（XML 实体已反转义）；
/// - 响应既无欢迎语也无错误框（如被 Cloudflare 拦截）→ [`Error::Parse`]。
pub fn parse_sign_in(body: &str) -> Result<String, Error> {
    if let Some(caps) = NAME_PATTERN.captures(body) {
        return Ok(crate::utils::unescape_xml(caps[1].trim()));
    }
    if let Some(caps) = ERROR_PATTERN.captures(body) {
        let msg = caps
            .get(1)
            .or_else(|| caps.get(2))
            .map(|m| crate::utils::unescape_xml(m.as_str().trim()))
            .unwrap_or_default();
        return Err(Error::Protocol(msg));
    }
    Err(Error::parse(
        "sign in response",
        "neither welcome nor error block found (blocked by Cloudflare?)",
        crate::error::snippet(body, 512),
    ))
}

#[cfg(test)]
mod tests {
    use super::parse_sign_in;

    #[test]
    fn success_yields_display_name() {
        let html = r#"<div id="userlinks"><p>You are now logged in as: EhFan_01</p></div>"#;
        assert_eq!(parse_sign_in(html).unwrap(), "EhFan_01");
    }

    #[test]
    fn error_block_yields_protocol_error() {
        let html = r#"<div><h4>The error returned was:</h4>
            <p>Sorry, the password was wrong</p></div>"#;
        let err = parse_sign_in(html).unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Protocol(m) if m.contains("password was wrong"))
        );
    }

    #[test]
    fn postcolor_error_yields_protocol_error() {
        let html = r#"<span class="postcolor">You must enter a captcha</span>"#;
        let err = parse_sign_in(html).unwrap_err();
        assert!(matches!(err, crate::error::Error::Protocol(m) if m.contains("captcha")));
    }

    #[test]
    fn unrelated_page_is_parse_error() {
        let err = parse_sign_in("<html>Just an error page</html>").unwrap_err();
        assert!(matches!(err, crate::error::Error::Parse { .. }));
    }
}
