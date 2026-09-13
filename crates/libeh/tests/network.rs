//! 真实网络冒烟测试。
//!
//! 默认跳过：设置 `EH_NETWORK_TESTS=1`（并按需配置 `.env` 中的
//! `EH_PROXY`）后运行 `cargo test --test network -- --ignored`。

use libeh::client::{client::EhClient, config::EhClientConfig};
use libeh::dto::keyword::Keyword;

fn gate() {
    if std::env::var("EH_NETWORK_TESTS").as_deref() != Ok("1") {
        eprintln!("skipped: set EH_NETWORK_TESTS=1 to enable network tests");
        panic!("network test gated");
    }
}

#[tokio::test]
#[ignore = "real network; enable with EH_NETWORK_TESTS=1"]
async fn search_smoke() {
    gate();
    dotenvy::dotenv().ok();
    let config = EhClientConfig::env().expect("valid env config");
    let client = EhClient::try_new(config).expect("client builds");
    let result = client
        .search_parsed(
            vec![
                Keyword::Artist("simon".into()),
                Keyword::Language("chinese".into()),
            ],
            None,
        )
        .await
        .expect("search should succeed");
    assert!(!result.gallery_info_list.is_empty(), "expected results");
}
