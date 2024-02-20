use serde::{Deserialize, Serialize};

/** 搜索结果的类型 */
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Category {
    Unknown = 0,
    Misc = 1,
    Doujinshi = 2,
    Manga = 4,
    ArtistCG = 8,
    GameCG = 16,
    ImageSet = 32,
    Cosplay = 64,
    // AsianPorn = 128,
    NonH = 256,
    Western = 512,
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
            "game cg" => Category::GameCG,
            "image set" => Category::ImageSet,
            "cosplay" => Category::Cosplay,
            "non-h" => Category::NonH,
            "western" => Category::Western,
            _ => Category::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dto::{category::Category, site::Site};
    use crate::url::search::SearchBuilder;

    #[test]
    fn test_category() {
        let category = SearchBuilder::new(Site::Eh)
            .toggle_category(Category::Doujinshi)
            .category();
        assert_eq!(category, 2);

        let category = SearchBuilder::new(Site::Eh).category();
        assert_eq!(category, 0);

        let category = SearchBuilder::new(Site::Eh)
            .toggle_category(Category::ArtistCG)
            .category();
        assert_eq!(category, 8);

        let category = SearchBuilder::new(Site::Eh)
            .toggle_category(Category::ArtistCG)
            .toggle_category(Category::ArtistCG)
            .category();
        assert_eq!(category, 0);

        let category = SearchBuilder::new(Site::Eh)
            .mask_all_categories()
            .toggle_category(Category::Doujinshi)
            .category();
        assert_eq!(category, 1021);

        let category = SearchBuilder::new(Site::Eh)
            .mask_all_categories()
            .toggle_category(Category::Doujinshi)
            .toggle_category(Category::Misc)
            .category();
        assert_eq!(category, 1020);
    }
}
