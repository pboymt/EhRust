//! 画廊分类枚举及其与站点分类位掩码（`f_cats`）之间的转换。
//!
//! E-Hentai 共有 10 个真实分类，每个对应 `f_cats` 排除掩码中的一个位；
//! 位定义与站点一致：Misc=`0x1`、Doujinshi=`0x2` … Western=`0x200`。
//!
//! # 位值与掩码语义
//!
//! `f_cats` 是**排除**掩码：置位的分类不出现在搜索结果里。
//! [`From<Category> for u16`] 只对这 10 个真实分类产出位值；
//! [`Category::None`](crate::dto::gallery::category::Category::None) / [`Category::Private`](crate::dto::gallery::category::Category::Private) / [`Category::Unknown`](crate::dto::gallery::category::Category::Unknown)
//! 不是合法的掩码位，转换为 `0`（写入 URL 时等效于未设置）。
//! `All`（`0x3FF`）表示"全部真实分类"，也不应作为排除掩码使用。
//!
//! ```rust
//! use libeh::dto::gallery::category::Category;
//!
//! assert_eq!(u16::from(Category::Doujinshi), 0x2);
//! assert_eq!(u16::from(Category::AsianPorn), 0x80);
//! assert_eq!(u16::from(Category::Unknown), 0); // 非法掩码位
//!
//! // 字符串解析覆盖站点页面上的各种写法
//! assert_eq!(Category::from("Artist CG".to_string()), Category::ArtistCG);
//! assert_eq!(Category::from("Artist CG Sets".to_string()), Category::ArtistCG);
//! assert_eq!(Category::from("Non-H".to_string()), Category::NonH);
//! ```

use serde::{Deserialize, Serialize};

/// 画廊的大分类。
///
/// 各真实分类的判别值即站点 `f_cats` 掩码位；`None`/`Private`/`Unknown`
/// 为解析层的占位语义值，见[模块文档](self)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    /// 无类型（解析失败或页面未标注时的占位值）。
    None = -1,
    /// 杂项 `0x1`
    Misc = 1,
    /// 同人志 `0x2`
    Doujinshi = 2,
    /// 漫画 `0x4`
    Manga = 4,
    /// 艺术家 CG `0x8`
    ArtistCG = 8,
    /// 游戏 CG `0x10`
    GameCG = 16,
    /// 图集 `0x20`
    ImageSet = 32,
    /// 角色扮演 `0x40`
    Cosplay = 64,
    /// 亚洲真人 `0x80`
    AsianPorn = 128,
    /// 无 H 内容 `0x100`
    NonH = 256,
    /// 西方作品 `0x200`
    Western = 512,
    /// 全部真实分类的掩码 `0x3FF`（不是合法的排除位）。
    All = 1023,
    /// 私有画廊（仅对上传者可见；不是合法的排除位）。
    Private = 1024,
    /// 未知类型（不是合法的排除位）。
    Unknown = 2048,
}

impl From<Category> for u16 {
    /// 转换为站点 `f_cats` 掩码位。
    ///
    /// 只有 10 个真实分类产出非零位值；[`Category::None`]、
    /// [`Category::Private`]、[`Category::Unknown`] 不是合法的排除位，
    /// 转换为 `0`；[`Category::All`] 亦返回 `0`——"全部分类"应通过
    /// "不设置 `f_cats`"表达（[`crate::url::search::SearchBuilder`] 的默认行为）。
    fn from(category: Category) -> u16 {
        match category {
            Category::Misc => 1,
            Category::Doujinshi => 2,
            Category::Manga => 4,
            Category::ArtistCG => 8,
            Category::GameCG => 16,
            Category::ImageSet => 32,
            Category::Cosplay => 64,
            Category::AsianPorn => 128,
            Category::NonH => 256,
            Category::Western => 512,
            Category::All | Category::None | Category::Private | Category::Unknown => 0,
        }
    }
}

impl From<String> for Category {
    /// 解析站点页面上渲染的分类文本（大小写不敏感，兼容新旧多种写法）。
    ///
    /// 无法识别时返回 [`Category::None`]。
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "misc" => Category::Misc,
            "doujinshi" => Category::Doujinshi,
            "manga" => Category::Manga,
            "artist cg" | "artist cg sets" | "artistcg" => Category::ArtistCG,
            "game cg" | "game cg sets" | "gamecg" => Category::GameCG,
            "image set" | "image sets" | "imageset" => Category::ImageSet,
            "cosplay" => Category::Cosplay,
            "asian porn" | "asianporn" => Category::AsianPorn,
            "non-h" => Category::NonH,
            "western" => Category::Western,
            "private" => Category::Private,
            "unknown" => Category::Unknown,
            _ => Category::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Category;

    #[test]
    fn mask_bits_match_site() {
        assert_eq!(u16::from(Category::Misc), 0x1);
        assert_eq!(u16::from(Category::Doujinshi), 0x2);
        assert_eq!(u16::from(Category::Manga), 0x4);
        assert_eq!(u16::from(Category::ArtistCG), 0x8);
        assert_eq!(u16::from(Category::GameCG), 0x10);
        assert_eq!(u16::from(Category::ImageSet), 0x20);
        assert_eq!(u16::from(Category::Cosplay), 0x40);
        assert_eq!(u16::from(Category::AsianPorn), 0x80);
        assert_eq!(u16::from(Category::NonH), 0x100);
        assert_eq!(u16::from(Category::Western), 0x200);
    }

    #[test]
    fn invalid_mask_values_are_zero() {
        assert_eq!(u16::from(Category::All), 0);
        assert_eq!(u16::from(Category::Private), 0);
        assert_eq!(u16::from(Category::Unknown), 0);
        assert_eq!(u16::from(Category::None), 0);
    }

    #[test]
    fn parse_site_texts() {
        assert_eq!(Category::from("MISC".to_string()), Category::Misc);
        assert_eq!(
            Category::from("artist cg sets".to_string()),
            Category::ArtistCG
        );
        assert_eq!(
            Category::from("Asian Porn".to_string()),
            Category::AsianPorn
        );
        assert_eq!(Category::from("non-h".to_string()), Category::NonH);
        assert_eq!(Category::from("private".to_string()), Category::Private);
        assert_eq!(Category::from("???".to_string()), Category::None);
    }
}
