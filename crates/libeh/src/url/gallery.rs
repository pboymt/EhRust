//! 画廊 URL 的构建与解析。
//!
//! 画廊 URL 模板：`https://(e-hentai|exhentai|lofi.e-hentai).org/(g|mpv)/{gid}/{token}/`，
//! 详情页可带 `?p={预览页}` 查询参数。
//!
//! [`GalleryBuilder`] 双向可用：
//!
//! - **解析**（[`GalleryBuilder::parse`] / [`GalleryBuilder::parse_with`]）：
//!   从任意文本中提取 gid 与 token。strict 模式要求完整合法 URL
//!   （与站点 API 文档一致），宽松模式从混合文本中抽取
//!   `gid/token` 片段（对齐 EhViewer `GalleryDetailUrlParser` 的双模式）；
//! - **构建**（[`GalleryBuilder::url`] / [`GalleryBuilder::eh_url`] /
//!   [`GalleryBuilder::ex_url`]）：生成对应站点的详情页 URL（`url()` 使用解析时保留的站点）。
//!
//! # 示例
//!
//! ```rust
//! use libeh::url::gallery::GalleryBuilder;
//!
//! let builder = GalleryBuilder::parse("https://e-hentai.org/g/530350/8b3c7e4a21/".into()).unwrap();
//! assert_eq!(builder.gid, 530350);
//! assert_eq!(builder.token, "8b3c7e4a21");
//!
//! // 宽松模式可从零散文本中提取
//! let loose = GalleryBuilder::parse_with("530350/8b3c7e4a21".into(), false).unwrap();
//! assert_eq!(loose.gid, 530350);
//!
//! // token 必须是 10 位十六进制
//! assert!(GalleryBuilder::parse_with("530350/8b3c7e4a211".into(), false).is_err());
//!
//! // 构建回 URL（带预览页码）
//! let mut builder2 = GalleryBuilder::new(530350, "8b3c7e4a21");
//! builder2.page(2);
//! assert_eq!(builder2.eh_url().as_str(), "https://e-hentai.org/g/530350/8b3c7e4a21/?p=2");
//! ```

use std::sync::LazyLock;

use regex::Regex;
use reqwest::Url;

use crate::dto::site::Site;
use crate::error::Error;

/// strict 模式：完整 URL，限定三个站点域名与 `g`/`mpv` 路径。
static URL_STRICT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"https?://(?:e-hentai\.org|exhentai\.org|lofi\.e-hentai\.org)/(?:g|mpv)/",
        r"(?<gid>\d+)/(?<token>[0-9a-f]{10})"
    ))
    .expect("constant regex")
});

/// 宽松模式：任意文本中的 `gid/token` 片段（token 固定 10 位十六进制）。
static URL_LOOSE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?<gid>\d+)/(?<token>[0-9a-f]{10})(?:[^0-9a-f]|$)").expect("constant regex")
});

/// 画廊 URL 的构建器与解析结果。
///
/// `p` 为详情页的预览页码（0 表示第一页，不产生 `?p=` 参数）。
/// 解析得到的构建器会**保留原 URL 的站点**（[`GalleryBuilder::url`]）；
/// [`GalleryBuilder::eh_url`] / [`GalleryBuilder::ex_url`] 用于显式跨站构造。
#[derive(Debug, Clone)]
pub struct GalleryBuilder {
    /// 画廊 ID。
    pub gid: i64,
    /// 画廊版本令牌（10 位十六进制）。
    pub token: String,
    /// 详情页预览页码（0 基；0 时不写入 `?p=`）。
    pub p: i64,
    /// 画廊所在站点：解析时取自原 URL，手工构造时默认表站。
    pub site: Site,
}

impl GalleryBuilder {
    /// 以 gid 与 token 创建构建器（预览页码为 0，站点为表站）。
    #[must_use]
    pub fn new(gid: i64, token: &str) -> Self {
        Self {
            gid,
            token: token.to_string(),
            p: 0,
            site: Site::Eh,
        }
    }

    /// 设置详情页预览页码。
    pub fn page(&mut self, p: i64) -> &mut Self {
        self.p = p;
        self
    }

    /// 以 strict 模式解析画廊 URL。
    ///
    /// # Errors
    ///
    /// 文本不是合法画廊 URL 时返回 [`Error::Parse`]。
    pub fn parse(s: String) -> Result<Self, Error> {
        Self::parse_with(s, true)
    }

    /// 解析画廊 URL。
    ///
    /// `strict = true` 时要求完整的 `https://…/(g|mpv)/…` URL；
    /// `strict = false` 时允许从混合文本中提取 `gid/token` 片段
    /// （例如粘贴的分享文本）。
    ///
    /// # Errors
    ///
    /// 未匹配到画廊 URL 形态或 gid 非法时返回 [`Error::Parse`]。
    pub fn parse_with(s: String, strict: bool) -> Result<Self, Error> {
        let pattern = if strict {
            &*URL_STRICT_PATTERN
        } else {
            &*URL_LOOSE_PATTERN
        };
        let caps = pattern
            .captures(&s)
            .ok_or_else(|| Error::parse("gallery url", "not a gallery url", s.clone()))?;
        let gid = caps["gid"]
            .parse::<i64>()
            .map_err(|e| Error::parse("gallery url gid", e.to_string(), s.clone()))?;
        let site = Site::from(caps[0].to_string());
        Ok(Self {
            gid,
            token: caps["token"].to_string(),
            p: 0,
            site,
        })
    }

    /// 构建里站（exhentai.org）详情页 URL。
    #[must_use]
    pub fn ex_url(&self) -> Url {
        self.build_url(Site::Ex)
    }

    /// 构建表站（e-hentai.org）详情页 URL。
    #[must_use]
    pub fn eh_url(&self) -> Url {
        self.build_url(Site::Eh)
    }

    /// 构建画廊**原站点**的详情页 URL。
    ///
    /// 解析路径（[`GalleryBuilder::parse`]）保留原 URL 的站点；
    /// 手工构造（[`GalleryBuilder::new`]）默认表站。
    /// 抓取画廊详情应使用本方法——里站专属画廊在表站不可见。
    #[must_use]
    pub fn url(&self) -> Url {
        self.build_url(self.site)
    }

    /// 以指定站点构建详情页 URL（`p > 0` 时附加 `?p=` 查询参数）。
    fn build_url(&self, site: Site) -> Url {
        let mut url: Url = site.url().expect("known site");
        url.set_path(&format!("g/{}/{}/", self.gid, self.token));
        if self.p > 0 {
            url.set_query(Some(&format!("p={}", self.p)));
        }
        url
    }
}

#[cfg(test)]
mod tests {
    use super::GalleryBuilder;
    use crate::dto::site::Site;
    use crate::error::Error;

    /// 迁移自 EhViewer `GalleryDetailUrlParserTest` 的参数化用例表。
    #[test]
    fn parse_preserves_site() {
        // 复审回归：解析结果保留原 URL 站点，里站专属画廊不应被指到表站
        let ex = GalleryBuilder::parse("https://exhentai.org/g/123/abcd123456/".into()).unwrap();
        assert_eq!(ex.site, Site::Ex);
        assert!(ex.url().as_str().starts_with("https://exhentai.org/"));
        let eh = GalleryBuilder::parse("https://e-hentai.org/g/123/abcd123456/".into()).unwrap();
        assert_eq!(eh.site, Site::Eh);
        // 手工构造默认表站
        assert_eq!(GalleryBuilder::new(1, "abcd123456").site, Site::Eh);
    }

    #[test]
    fn parse_cases_from_ehviewer() {
        // (输入, strict, 期望: Some((gid, token)) 或 None)
        type Case = (&'static str, bool, Option<(i64, &'static str)>);
        let cases: &[Case] = &[
            (
                "https://e-hentai.org/g/530350/8b3c7e4a21/",
                true,
                Some((530350, "8b3c7e4a21")),
            ),
            (
                "https://exhentai.org/g/530350/8b3c7e4a21/",
                true,
                Some((530350, "8b3c7e4a21")),
            ),
            (
                "https://lofi.e-hentai.org/g/530350/8b3c7e4a21/",
                true,
                Some((530350, "8b3c7e4a21")),
            ),
            (
                "https://e-hentai.org/mpv/530350/8b3c7e4a21/",
                true,
                Some((530350, "8b3c7e4a21")),
            ),
            (
                "https://exhentai.org/mpv/530350/8b3c7e4a21/",
                true,
                Some((530350, "8b3c7e4a21")),
            ),
            (
                "https://lofi.e-hentai.org/mpv/530350/8b3c7e4a21/",
                true,
                Some((530350, "8b3c7e4a21")),
            ),
            (
                "https://e-hentai.org/mpv/530350/8b3c7e4a21/#page1",
                true,
                Some((530350, "8b3c7e4a21")),
            ),
            ("530350/8b3c7e4a21/#page1", true, None),
            ("530350/8b3c7e4a21", false, Some((530350, "8b3c7e4a21"))),
            ("g/530350/8b3c7e4a21", false, Some((530350, "8b3c7e4a21"))),
            ("530350b/8b3c7e4a21", false, None),
            ("530350/8b3c7e4a211", false, None),
        ];
        for (input, strict, expected) in cases {
            let result = GalleryBuilder::parse_with((*input).to_string(), *strict);
            match expected {
                Some((gid, token)) => {
                    let builder = result.unwrap_or_else(|e| panic!("{input:?} should parse: {e}"));
                    assert_eq!(builder.gid, *gid, "{input:?}");
                    assert_eq!(builder.token, *token, "{input:?}");
                }
                None => {
                    assert!(result.is_err(), "{input:?} should not parse");
                    assert!(matches!(result.unwrap_err(), Error::Parse { .. }));
                }
            }
        }
    }
}
