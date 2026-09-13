//! `ehrust` 命令行工具：用 libeh 搜索 E-Hentai/ExHentai 并打印结果。
//!
//! 配置来源（后者覆盖前者）：
//! 1. 环境变量（`EH_SITE`/`EH_PROXY`/`EH_AUTH_*`，可放 `.env`）；
//! 2. `--config <PATH>` 指定的 YAML 配置文件（缺省为 `config.yaml`，存在才加载）；
//! 3. 命令行参数（`--site`/`--proxy`，见 `--help`）。
//!
//! 输出对敏感字段脱敏（`ipb_pass_hash`/`igneous` 显示为 `****`）。
//!
//! # 示例
//!
//! ```console
//! $ ehrust search -k "artist:simon" -k "language:chinese"
//! $ ehrust search -k "female:big breasts" --dry-run    # 仅打印搜索 URL
//! $ ehrust gallery https://e-hentai.org/g/2791585/3e7e1c7107/
//! ```

use std::{fs::File, path::PathBuf};

use clap::Parser as _;
use libeh::client::{client::EhClient, config::EhClientConfig};
use libeh::dto::gallery::detail::GalleryDetail;
use libeh::dto::keyword::Keyword;
use libeh::error::Error;
use libeh::url::gallery::GalleryBuilder;

/// `ehrust` 命令行入口。
#[derive(Debug, clap::Parser)]
#[command(about = "E-Hentai/ExHentai CLI powered by libeh", version)]
struct Cli {
    /// YAML 配置文件路径（默认 config.yaml，存在才加载）。
    #[arg(
        short = 'c',
        long = "config",
        global = true,
        default_value = "config.yaml"
    )]
    config: PathBuf,
    /// 站点：eh / ex（覆盖配置文件与环境变量）。
    #[arg(short = 's', long = "site", global = true)]
    site: Option<String>,
    /// 代理 URL（如 socks5://127.0.0.1:7897，覆盖配置文件与环境变量）。
    #[arg(short = 'p', long = "proxy", global = true)]
    proxy: Option<String>,
    /// 子命令。
    #[command(subcommand)]
    command: Command,
}

/// 支持的子命令。
#[derive(Debug, clap::Subcommand)]
enum Command {
    /// 关键词搜索并打印结果。
    Search {
        /// 搜索关键词（可多次，支持 `female:xxx` 标签语法）。
        #[arg(short = 'k', long = "keyword")]
        keywords: Vec<String>,
        /// 连续抓取的页数（默认 1 页；使用分页迭代器逐页请求）。
        #[arg(long)]
        pages: Option<usize>,
        /// 打印原始搜索 URL 而不发请求（离线调试）。
        #[arg(long)]
        dry_run: bool,
    },
    /// 抓取并解析一个画廊详情页。
    Gallery {
        /// 画廊 URL（`/g/{gid}/{token}/`）。
        url: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    // 配置合并：环境变量 → 配置文件 → 命令行参数
    let mut config = EhClientConfig::env()?;
    if cli.config.exists() {
        let file = File::open(&cli.config)?;
        let file_config: EhClientConfig = serde_yaml::from_reader(file)?;
        merge_config(&mut config, file_config);
    }
    if let Some(site) = &cli.site {
        config.site = parse_site_arg(site)?;
    }
    if let Some(proxy) = &cli.proxy {
        config.proxy = Some(libeh::client::proxy::EhClientProxy::from_url(proxy)?);
    }

    match cli.command {
        Command::Search {
            keywords,
            pages,
            dry_run,
        } => run_search(config, keywords, pages, dry_run).await?,
        Command::Gallery { url } => run_gallery(config, url).await?,
    }
    Ok(())
}

/// 执行搜索子命令。
///
/// `pages > 1` 时使用分页迭代器连续抓取多页（仅支持数字分页的页面）。
async fn run_search(
    config: EhClientConfig,
    keywords: Vec<String>,
    pages: Option<usize>,
    dry_run: bool,
) -> Result<(), Error> {
    let keywords: Vec<Keyword> = keywords.into_iter().map(Keyword::from).collect();
    let builder = libeh::url::search::SearchBuilder::new(config.site).add_keywords(keywords);

    if dry_run {
        println!("{}", builder.build()?);
        return Ok(());
    }

    println!("Config: {config:?}");
    let client = EhClient::try_new(config)?;
    let max_pages = pages.unwrap_or(1).max(1);
    let mut pager = libeh::client::pagination::SearchPager::new(client, builder);
    let mut fetched = 0usize;
    while fetched < max_pages {
        let Some(result) = pager.next().await else {
            break;
        };
        let result = result?;
        fetched += 1;
        println!(
            "== page {}: pages={} next_page={} count={:?} skipped={}",
            fetched, result.pages, result.next_page, result.result_count, result.skipped_rows
        );
        for gallery_info in result.gallery_info_list {
            println!("{gallery_info:?}");
        }
    }
    Ok(())
}

/// 执行画廊子命令：抓取详情页并打印摘要。
///
/// 使用解析自 URL 的原站点（里站专属画廊在表站不可见）。
async fn run_gallery(config: EhClientConfig, url: String) -> Result<(), Error> {
    let builder = GalleryBuilder::parse(url)?;
    println!("Config: {config:?}");
    let client = EhClient::try_new(config)?;
    let html = client.get_html(builder.url()).await?;
    let detail = GalleryDetail::parse(html)?;
    println!("title: {}", detail.info.title);
    println!("category: {:?}", detail.info.category);
    println!("pages: {}", detail.info.pages);
    println!(
        "rating: {} ({} votes)",
        detail.info.rating, detail.rating_count
    );
    println!("torrents: {}", detail.torrent_count);
    println!("tags: {:?}", detail.info.tags);
    println!("comments: {}", detail.comments.len());
    Ok(())
}

/// 用配置文件的**非默认**字段覆盖环境变量配置。
///
/// YAML 中未出现的字段（`site: un` 或缺省 proxy/auth）不覆盖已有值。
fn merge_config(base: &mut EhClientConfig, file: EhClientConfig) {
    if file.site != libeh::dto::site::Site::Un {
        base.site = file.site;
    }
    if file.proxy.is_some() {
        base.proxy = file.proxy;
    }
    if file.auth.is_some() {
        base.auth = file.auth;
    }
}

/// 解析 `--site` 参数（`eh`/`ex` 或完整域名）。
fn parse_site_arg(raw: &str) -> Result<libeh::dto::site::Site, Error> {
    Ok(match raw.trim().to_ascii_lowercase().as_str() {
        "eh" | "e-hentai.org" => libeh::dto::site::Site::Eh,
        "ex" | "exhentai.org" => libeh::dto::site::Site::Ex,
        other => {
            return Err(Error::Config(format!(
                "unsupported site {other:?}; expected \"eh\" or \"ex\""
            )))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_site_arg, Cli};
    use clap::Parser as _;

    #[test]
    fn site_arg_parsing() {
        assert!(matches!(
            parse_site_arg("eh").unwrap(),
            libeh::dto::site::Site::Eh
        ));
        assert!(matches!(
            parse_site_arg("EX").unwrap(),
            libeh::dto::site::Site::Ex
        ));
        assert!(parse_site_arg("nope").is_err());
    }

    #[test]
    fn cli_parses_search_args() {
        let cli = Cli::try_parse_from([
            "ehrust",
            "search",
            "-k",
            "artist:simon",
            "-k",
            "language:chinese",
        ])
        .unwrap();
        match cli.command {
            super::Command::Search { keywords, .. } => {
                assert_eq!(keywords, vec!["artist:simon", "language:chinese"]);
            }
            _ => panic!("expected search subcommand"),
        }
    }
}
