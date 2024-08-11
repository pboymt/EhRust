use serde::{Deserialize, Serialize};

/// 画廊的大分类
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Category {
    /// 无类型
    None = -1,
    /// 杂项        0b0000000001 | 0x1
    Misc = 1,
    /// 同人志      0b0000000010 | 0x2
    Doujinshi = 2,
    /// 漫画        0b0000000100 | 0x4
    Manga = 4,
    /// 艺术家 CG   0b0000001000 | 0x8
    ArtistCG = 8,
    /// 游戏 CG     0b0000010000 | 0x10
    GameCG = 16,
    /// 图集        0b0000100000 | 0x20
    ImageSet = 32,
    /// 角色扮演    0b0001000000 | 0x40
    Cosplay = 64,
    // AsianPorn = 128,
    /// 无 H 内容   0b0100000000 | 0x100
    NonH = 256,
    /// 西方作品    0b1000000000 | 0x200
    Western = 512,
    /// 全部类型    0b1111111110 | 0x3FF
    All = 1023,
    /// 私有画廊    0b0000000000 | 0x400
    Private = 1024,
    /// 未知类型    0b1100000000 | 0x800
    Unknown = 2048,
}

impl From<Category> for u16 {
    fn from(category: Category) -> u16 {
        match category {
            Category::Misc => 1,
            Category::Doujinshi => 2,
            Category::Manga => 4,
            Category::ArtistCG => 8,
            Category::GameCG => 16,
            Category::ImageSet => 32,
            Category::Cosplay => 64,
            Category::NonH => 256,
            Category::Western => 512,
            Category::All => 1023,
            Category::Private => 1024,
            Category::Unknown => 2048,
            _ => 0,
        }
    }
}

impl From<String> for Category {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "misc" => Category::Misc,
            "doujinshi" => Category::Doujinshi,
            "manga" => Category::Manga,
            "artist cg" => Category::ArtistCG,
            "artist cg sets" => Category::ArtistCG,
            "artistcg" => Category::ArtistCG,
            "game cg" => Category::GameCG,
            "game cg sets" => Category::GameCG,
            "gamecg" => Category::GameCG,
            "image set" => Category::ImageSet,
            "image sets" => Category::ImageSet,
            "imageset" => Category::ImageSet,
            "cosplay" => Category::Cosplay,
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
    use crate::dto::{gallery::category::Category, site::Site};
    use crate::url::search::SearchBuilder;

    #[test]
    /// 测试不同搜索条件下的类别值
    fn test_category() {
        // 初始化一个搜索构建器，针对 Eh 网站，切换到 Doujinshi 类别，并获取类别值
        let category = SearchBuilder::new(Site::Eh)
            .toggle_category(Category::Doujinshi)
            .category();
        // 断言类别值为 2
        assert_eq!(category, 2);

        // 初始化一个搜索构建器，针对Eh网站，不切换类别，并获取类别值
        let category = SearchBuilder::new(Site::Eh).category();
        // 断言类别值为 0
        assert_eq!(category, 0);

        // 初始化一个搜索构建器，针对 Eh 网站，切换到 ArtistCG 类别，并获取类别值
        let category = SearchBuilder::new(Site::Eh)
            .toggle_category(Category::ArtistCG)
            .category();
        // 断言类别值为 8
        assert_eq!(category, 8);

        // 初始化一个搜索构建器，针对 Eh 网站，切换到 ArtistCG 类别，再切换回 ArtistCG 类别，并获取类别值
        let category = SearchBuilder::new(Site::Eh)
            .toggle_category(Category::ArtistCG)
            .toggle_category(Category::ArtistCG)
            .category();
        // 断言类别值为 0，说明切换回同一类别会取消该类别
        assert_eq!(category, 0);

        // 初始化一个搜索构建器，针对 Eh 网站，屏蔽所有类别，切换到 Doujinshi 类别，并获取类别值
        let category = SearchBuilder::new(Site::Eh)
            .mask_all_categories()
            .toggle_category(Category::Doujinshi)
            .category();
        // 断言类别值为 1021，说明在屏蔽所有类别后再选择特定类别得到的结果
        assert_eq!(category, 1021);

        // 初始化一个搜索构建器，针对 Eh 网站，屏蔽所有类别，切换到 Doujinshi 和 Misc 类别，并获取类别值
        let category = SearchBuilder::new(Site::Eh)
            .mask_all_categories()
            .toggle_category(Category::Doujinshi)
            .toggle_category(Category::Misc)
            .category();
        // 断言类别值为 1020，说明在屏蔽所有类别后再选择多个特定类别得到的结果
        assert_eq!(category, 1020);
    }
}
