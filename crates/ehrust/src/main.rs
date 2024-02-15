use std::{fs::File, path::Path};

use libeh::{
    client::{client::EhClient, config::EhClientConfig},
    dto::{keyword::Keyword, search_result::SearchResult},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = open_config().await?;
    println!("Config: {:?}", config);
    let client = EhClient::new(config);
    let res = client
        .search(
            vec![
                Keyword::Artist("simon".into()),
                Keyword::Language("chinese".into()),
            ],
            None,
        )
        .await?;
    let result = SearchResult::parse(res)?;
    for gallery_info in result.gallery_info_list {
        println!("{:?}", gallery_info);
    }

    Ok(())
}

async fn open_config() -> Result<EhClientConfig, Box<dyn std::error::Error>> {
    let config_path = Path::new("config.yaml");
    let config = if config_path.exists() {
        let file = File::open(config_path)?;
        let conf: EhClientConfig = serde_yaml::from_reader(file)?;
        conf
    } else {
        let conf = EhClientConfig::default();
        conf
    };
    Ok(config)
}
