use libeh::url::{
    search::SearchBuilder,
    enums::{Category, Keyword, Offset, Site},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    let builder = SearchBuilder::new(Site::Eh);
    let builder = builder
        .offset(Offset::Prev(1, None))
        .mask_all_categories()
        .toggle_category(Category::Doujinshi)
        .enable_advanced_search()
        .keyword(Keyword::Female("living clothes".to_string()));
    let url = builder.build().unwrap();
    println!("url: {}", url.to_string());

    Ok(())
}
