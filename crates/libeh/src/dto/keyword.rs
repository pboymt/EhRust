//! 搜索关键词：各类标签的枚举及其与站点搜索语法之间的转换。
//!
//! E-Hentai 的搜索框接受形如 `female:big breasts` 的标签语法，
//! 其中冒号前的部分是**命名空间**（namespace），冒号后是标签值。
//! 本模块把每个命名空间建模为 [`Keyword`](crate::dto::keyword::Keyword) 的一个变体，
//! 并负责与站点查询语法（写入 `f_search` 参数的字符串）互转。
//!
//! # 命名空间对照表
//!
//! | 站点前缀 | 缩写 | 变体 | 含义 |
//! |---|---|---|---|
//! | （无） | — | [`Keyword::Normal`](crate::dto::keyword::Keyword::Normal) | 普通短语（子串匹配） |
//! | `language:` | `l:` | [`Keyword::Language`](crate::dto::keyword::Keyword::Language) | 作品语言 |
//! | `parody:` | `p:` | [`Keyword::Parody`](crate::dto::keyword::Keyword::Parody) | 同人作品模仿的原始作品 |
//! | `character:` | `c:` | [`Keyword::Character`](crate::dto::keyword::Keyword::Character) | 作品中出现的角色 |
//! | `artist:` | `a:` | [`Keyword::Artist`](crate::dto::keyword::Keyword::Artist) | 绘画作者/写手 |
//! | `cosplayer:` | `cos:` | [`Keyword::Cosplayer`](crate::dto::keyword::Keyword::Cosplayer) | 角色扮演者 |
//! | `group:` | `g:` | [`Keyword::Group`](crate::dto::keyword::Keyword::Group) | 制作社团或公司 |
//! | `female:` | `f:` | [`Keyword::Female`](crate::dto::keyword::Keyword::Female) | 女性角色相关标签 |
//! | `male:` | `m:` | [`Keyword::Male`](crate::dto::keyword::Keyword::Male) | 男性角色相关标签 |
//! | `mixed:` | `x:` | [`Keyword::Mixed`](crate::dto::keyword::Keyword::Mixed) | 两性/中性标签 |
//! | `other:` | `o:` | [`Keyword::Other`](crate::dto::keyword::Keyword::Other) | 其他已确认技术标签 |
//! | `reclass:` | `r:` | [`Keyword::Reclass`](crate::dto::keyword::Keyword::Reclass) | 重新分类候选标签 |
//! | `temp:` | `temp:` | [`Keyword::Temp`](crate::dto::keyword::Keyword::Temp) | 尚未正式加入标签系统的标签 |
//! | `uploader:` | — | [`Keyword::Uploader`](crate::dto::keyword::Keyword::Uploader) | 上传者名称 |
//!
//! # 生成语法
//!
//! [`Display`](std::fmt::Display) 生成的片段遵循站点规则：
//!
//! - [`Keyword::Normal`](crate::dto::keyword::Keyword::Normal) 输出为裸词（子串匹配），**不加引号**；
//! - 其余命名空间输出为 `ns:"value$"`，`$` 后缀表示**标签精确匹配**
//!   （无 `$` 则为标签前缀匹配）；
//! - 值中的 `"` 会被剔除——站点搜索语法不支持引号转义，
//!   保留它只会产生无法命中的查询。
//!
//! # 解析规则
//!
//! [`FromStr`](std::str::FromStr) 按**第一个**冒号切分命名空间与值，因此值本身允许再含冒号，
//! 如 `parody:re:zero kara hajimeru isekai seikatsu`。解析永远不会失败：
//! 命名空间无法识别时回退为 [`Keyword::Normal`](crate::dto::keyword::Keyword::Normal)，因此 `Err` 类型为 [`Infallible`](std::convert::Infallible)。
//!
//! # 示例
//!
//! ```rust
//! use libeh::dto::keyword::Keyword;
//! use std::str::FromStr;
//!
//! // 命名空间精确匹配，值中的冒号不会破坏解析
//! let tag = Keyword::from_str("parody:re:zero kara hajimeru isekai seikatsu").unwrap();
//! assert_eq!(tag, Keyword::Parody("re:zero kara hajimeru isekai seikatsu".into()));
//! assert_eq!(tag.to_string(), r#"parody:"re:zero kara hajimeru isekai seikatsu$""#);
//!
//! // 缩写前缀、引号与 $ 后缀在解析时都被剥掉
//! assert_eq!(
//!     Keyword::from_str(r#"l:"chinese$""#).unwrap(),
//!     Keyword::Language("chinese".into())
//! );
//!
//! // 未知命名空间回退为普通关键词，而不是报错
//! assert_eq!(
//!     Keyword::from_str("weird:thing").unwrap(),
//!     Keyword::Normal("weird:thing".into())
//! );
//! ```

use std::convert::Infallible;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// 搜索关键词。
///
/// 详见[模块文档](self)中的命名空间对照表与生成语法。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Keyword {
    /// 一般的搜索关键词（子串匹配，生成时不加引号）。
    Normal(String),
    /// TAG 作品的语言（`language:` / `l:`）。
    Language(String),
    /// TAG 同人作品模仿的原始作品（`parody:` / `p:`）。
    Parody(String),
    /// TAG 作品中出现的角色（`character:` / `c:`）。
    Character(String),
    /// TAG 绘画作者/写手（`artist:` / `a:`）。
    Artist(String),
    /// TAG 角色扮演者（`cosplayer:` / `cos:`）。
    Cosplayer(String),
    /// TAG 制作社团或公司（`group:` / `g:`）。
    Group(String),
    /// TAG 女性角色相关的恋物标签（`female:` / `f:`）。
    Female(String),
    /// TAG 男性角色相关的恋物标签（`male:` / `m:`）。
    Male(String),
    /// TAG 两性/中性的恋物标签（`mixed:` / `x:`）。
    Mixed(String),
    /// TAG 其他已确认的技术标签（`other:` / `o:`）。
    Other(String),
    /// TAG 重新分类候选标签（`reclass:` / `r:`）。
    Reclass(String),
    /// TAG 尚未正式加入 E-Hentai 标签系统的标签（`temp:`）。
    Temp(String),
    /// TAG 上传者名称（`uploader:`，无缩写）。
    Uploader(String),
}

/// 剥掉值两侧的引号与尾部的 `$` 精确匹配后缀。
fn strip_syntax(value: &str) -> String {
    let mut v = value.trim();
    if v.len() >= 2 && v.starts_with('"') && v.ends_with('"') {
        v = v[1..v.len() - 1].trim();
    }
    let v = v.strip_suffix('$').unwrap_or(v);
    v.trim().to_string()
}

/// 剔除站点语法不支持的双重引号并去除首尾空白。
fn sanitize(value: &str) -> String {
    value.trim().replace('"', "")
}

impl Keyword {
    /// 返回该关键词对应的站点命名空间前缀（如 `"female"`）；
    /// [`Keyword::Normal`] 没有命名空间，返回 `None`。
    #[must_use]
    pub fn namespace(&self) -> Option<&'static str> {
        match self {
            Keyword::Normal(_) => None,
            Keyword::Language(_) => Some("language"),
            Keyword::Parody(_) => Some("parody"),
            Keyword::Character(_) => Some("character"),
            Keyword::Artist(_) => Some("artist"),
            Keyword::Cosplayer(_) => Some("cosplayer"),
            Keyword::Group(_) => Some("group"),
            Keyword::Female(_) => Some("female"),
            Keyword::Male(_) => Some("male"),
            Keyword::Mixed(_) => Some("mixed"),
            Keyword::Other(_) => Some("other"),
            Keyword::Reclass(_) => Some("reclass"),
            Keyword::Temp(_) => Some("temp"),
            Keyword::Uploader(_) => Some("uploader"),
        }
    }
}

impl fmt::Display for Keyword {
    /// 生成为站点搜索语法片段（写入 `f_search` 参数）。
    ///
    /// 规则见[模块文档](self)#生成语法：`Normal` 输出裸词；
    /// 其余命名空间输出 `ns:"value$"`（`$` = 精确匹配）；
    /// 值中的 `"` 一律剔除。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // 普通关键词按子串匹配，不引用、不加 $ 后缀
            Keyword::Normal(v) => f.write_str(&sanitize(v)),
            // 命名空间标签使用精确匹配（$ 后缀）
            Keyword::Language(v) => write!(f, r#"language:"{}$""#, sanitize(v)),
            Keyword::Parody(v) => write!(f, r#"parody:"{}$""#, sanitize(v)),
            Keyword::Character(v) => write!(f, r#"character:"{}$""#, sanitize(v)),
            Keyword::Artist(v) => write!(f, r#"artist:"{}$""#, sanitize(v)),
            Keyword::Cosplayer(v) => write!(f, r#"cosplayer:"{}$""#, sanitize(v)),
            Keyword::Group(v) => write!(f, r#"group:"{}$""#, sanitize(v)),
            Keyword::Female(v) => write!(f, r#"female:"{}$""#, sanitize(v)),
            Keyword::Male(v) => write!(f, r#"male:"{}$""#, sanitize(v)),
            Keyword::Mixed(v) => write!(f, r#"mixed:"{}$""#, sanitize(v)),
            Keyword::Other(v) => write!(f, r#"other:"{}$""#, sanitize(v)),
            Keyword::Reclass(v) => write!(f, r#"reclass:"{}$""#, sanitize(v)),
            Keyword::Temp(v) => write!(f, r#"temp:"{}$""#, sanitize(v)),
            // 上传者搜索本身就是精确的，不需要 $ 后缀
            Keyword::Uploader(v) => write!(f, r#"uploader:"{}""#, sanitize(v)),
        }
    }
}

impl From<Keyword> for String {
    fn from(keyword: Keyword) -> String {
        keyword.to_string()
    }
}

impl FromStr for Keyword {
    /// 解析永远不会失败：未知命名空间回退为 [`Keyword::Normal`]。
    type Err = Infallible;

    /// 解析站点搜索语法片段。
    ///
    /// 规则见[模块文档](self)#解析规则。输入两侧空白、值两侧引号、
    /// 命名空间标签的 `$` 后缀均会被剥离。
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let Some((ns, value)) = s.split_once(':') else {
            return Ok(Keyword::Normal(sanitize(s)));
        };
        let value = strip_syntax(value);
        match ns.trim().to_ascii_lowercase().as_str() {
            "language" | "l" => Ok(Keyword::Language(value)),
            "parody" | "p" => Ok(Keyword::Parody(value)),
            "character" | "c" => Ok(Keyword::Character(value)),
            "artist" | "a" => Ok(Keyword::Artist(value)),
            "cosplayer" | "cos" => Ok(Keyword::Cosplayer(value)),
            "group" | "g" => Ok(Keyword::Group(value)),
            "female" | "f" => Ok(Keyword::Female(value)),
            "male" | "m" => Ok(Keyword::Male(value)),
            "mixed" | "x" => Ok(Keyword::Mixed(value)),
            "other" | "o" => Ok(Keyword::Other(value)),
            "reclass" | "r" => Ok(Keyword::Reclass(value)),
            "temp" => Ok(Keyword::Temp(value)),
            "uploader" => Ok(Keyword::Uploader(value)),
            // 未知命名空间：保留原文作为普通关键词
            _ => Ok(Keyword::Normal(sanitize(s))),
        }
    }
}

impl From<String> for Keyword {
    /// 同 [`FromStr`]，无法识别时回退 [`Keyword::Normal`]，永不失败。
    fn from(s: String) -> Keyword {
        // FromStr 的 Err 类型为 Infallible，此分支不可达
        match Keyword::from_str(&s) {
            Ok(keyword) => keyword,
            Err(e) => match e {},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multi_colon_tag_value() {
        // 回归：标签值本身含冒号（E-Hentai 常见，如 re:zero 系列作品）
        let tag = Keyword::from_str("parody:re:zero kara hajimeru isekai seikatsu").unwrap();
        assert_eq!(
            tag,
            Keyword::Parody("re:zero kara hajimeru isekai seikatsu".into())
        );
    }

    #[test]
    fn abbreviations_and_syntax_markers() {
        assert_eq!(
            Keyword::from_str("l:chinese").unwrap(),
            Keyword::Language("chinese".into())
        );
        assert_eq!(
            Keyword::from_str(r#"l:"chinese$""#).unwrap(),
            Keyword::Language("chinese".into())
        );
        assert_eq!(
            Keyword::from_str("cos:foo").unwrap(),
            Keyword::Cosplayer("foo".into())
        );
        assert_eq!(
            Keyword::from_str("uploader:someone").unwrap(),
            Keyword::Uploader("someone".into())
        );
    }

    #[test]
    fn unknown_namespace_falls_back() {
        assert_eq!(
            Keyword::from_str("weird:thing").unwrap(),
            Keyword::Normal("weird:thing".into())
        );
    }

    #[test]
    fn plain_word_is_normal() {
        assert_eq!(
            Keyword::from_str("hello world").unwrap(),
            Keyword::Normal("hello world".into())
        );
    }

    #[test]
    fn display_syntax() {
        assert_eq!(Keyword::Normal("big".into()).to_string(), "big");
        assert_eq!(
            Keyword::Female("big breasts".into()).to_string(),
            r#"female:"big breasts$""#
        );
        assert_eq!(
            Keyword::Uploader("someone".into()).to_string(),
            r#"uploader:"someone""#
        );
        // 值中的引号被剔除，保证生成的查询合法
        assert_eq!(
            Keyword::Artist(r#"a "quoted" name"#.into()).to_string(),
            r#"artist:"a quoted name$""#
        );
    }

    #[test]
    fn display_parse_roundtrip() {
        for raw in [
            "big breasts",
            "parody:re:zero kara hajimeru isekai seikatsu",
            "l:chinese",
            "uploader:someone",
        ] {
            let k = Keyword::from_str(raw).unwrap();
            let rebuilt = Keyword::from_str(&k.to_string()).unwrap();
            assert_eq!(k, rebuilt, "roundtrip failed for {raw:?}");
        }
    }
}
