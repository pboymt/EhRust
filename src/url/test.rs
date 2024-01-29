#[test]
fn test_category() {
    use super::{
        search::SearchBuilder,
        enums::{Category, Site},
    };
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
