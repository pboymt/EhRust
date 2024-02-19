use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::utils::serde::{
    parse_float32_str, parse_int32_str, parse_int64_str, parse_keyword_strings,
    parse_option_int64_str, parse_unix_timestamp_str,
};

use super::keyword::Keyword;

/// 画廊 ID 及其令牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GIDListItem(i64, String);

impl GIDListItem {
    /// 新建画廊 ID 及其令牌
    pub fn new(gid: i64, token: &str) -> Self {
        GIDListItem(gid, token.into())
    }
}

impl From<String> for GIDListItem {
    fn from(value: String) -> Self {
        const URL_PATTERN: &'static str =
            r"^https://e[\-x]hentai.org/g/(?<gid>\d+)/(?<token>[a-f0-9]+)/?";
        let regex = regex::Regex::new(URL_PATTERN).unwrap();
        match regex.captures(&value) {
            Some(captures) => {
                let gid = captures["gid"].parse().unwrap();
                let token = captures["token"].to_string();
                GIDListItem(gid, token)
            }
            None => panic!("Invalid URL: {}", value),
        }
    }
}

/// 通过画廊 ID 及其令牌检索元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryMetadataRequest {
    /// 请求方法，应恒为 "gdata"
    pub method: String,
    /// 画廊 ID 及其令牌列表
    pub gidlist: Vec<GIDListItem>,
    /// 画廊元数据命名空间，应恒为 1，否则输出 tags 不包含命名空间
    pub namespace: i32,
}

impl GalleryMetadataRequest {
    /// 将包含画廊 ID 及其令牌的列表转换为请求数据
    pub fn new(gidlist: Vec<GIDListItem>) -> Self {
        Self {
            method: "gdata".to_string(),
            gidlist,
            namespace: 1,
        }
    }
}

/// 画廊种子数据
#[derive(Debug, Clone, Deserialize)]
pub struct GalleryTorrent {
    pub hash: String,
    #[serde(with = "parse_unix_timestamp_str")]
    pub added: DateTime<Utc>,
    pub name: String,
    #[serde(with = "parse_int64_str")]
    pub tsize: i64,
    #[serde(with = "parse_int64_str")]
    pub fsize: i64,
}

/// 画廊元数据，通过 API 请求获得
#[derive(Debug, Clone, Deserialize)]
pub struct GalleryMetadata {
    pub gid: i64,
    pub token: String,
    pub archiver_key: String,
    pub title: String,
    pub title_jpn: String,
    pub category: String,
    pub thumb: String,
    pub uploader: String,
    #[serde(with = "parse_unix_timestamp_str")]
    pub posted: DateTime<Utc>,
    #[serde(with = "parse_int32_str")]
    pub filecount: i32,
    pub filesize: i64,
    pub expunged: bool,
    #[serde(with = "parse_float32_str")]
    pub rating: f32,
    #[serde(with = "parse_int32_str")]
    pub torrentcount: i32,
    pub torrents: Vec<GalleryTorrent>,
    #[serde(with = "parse_keyword_strings")]
    pub tags: Vec<Keyword>,
    #[serde(with = "parse_option_int64_str")]
    pub parent_gid: Option<i64>,
    pub parent_key: Option<String>,
    #[serde(with = "parse_option_int64_str")]
    pub first_gid: Option<i64>,
    pub first_key: Option<String>,
}

/// 请求画廊元数据的响应数据
#[derive(Debug, Clone, Deserialize)]
pub struct GalleryMetadataResponse {
    pub gmetadata: Vec<GalleryMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GalleryMetadataError {
    pub gid: i64,
    pub error: String,
}

/// 页面列表，包含画廊 ID、页面令牌和页号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageListItem(i64, String, i32);

impl From<String> for PageListItem {
    fn from(value: String) -> Self {
        const URL_PATTERN: &'static str =
            r"^https://e[\-x]hentai.org/s/(?<ptoken>[a-f0-9]+)/(?<gid>\d+)-(?<pnum>\d+)/?";
        let regex = regex::Regex::new(URL_PATTERN).unwrap();
        match regex.captures(&value) {
            Some(captures) => {
                let ptoken = captures["ptoken"].to_string();
                let gid = captures["gid"].parse().unwrap();
                let pnum = captures["pnum"].parse().unwrap();
                PageListItem(gid, ptoken, pnum)
            }
            None => panic!("Invalid URL: {}", value),
        }
    }
}

/// 通过画廊 ID、页面令牌和页号反查画廊令牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryTokensRequest {
    /// 请求方法，应恒为 "gtoken"
    pub method: String,
    /// 页面列表
    pub pagelist: Vec<PageListItem>,
}

impl GalleryTokensRequest {
    /// 将包含页面列表的请求数据转换为请求数据
    pub fn new(pagelist: Vec<PageListItem>) -> Self {
        Self {
            method: "gtoken".to_string(),
            pagelist,
        }
    }
}

/// 画廊 ID 与令牌
#[derive(Debug, Clone, Deserialize)]
pub struct TokenListItem {
    pub gid: i64,
    pub token: String,
}

/// 反查画廊令牌的响应数据
#[derive(Debug, Clone, Deserialize)]
pub struct GalleryTokenResponse {
    pub tokenlist: Vec<TokenListItem>,
}

#[cfg(test)]
mod tests {
    use reqwest::Url;

    use crate::{
        client::{client::EhClient, config::EhClientConfig, proxy::EhClientProxy},
        dto::{
            api::{
                GIDListItem, GalleryMetadataRequest, GalleryMetadataResponse, GalleryTokenResponse,
                GalleryTokensRequest, PageListItem,
            },
            site::Site,
        },
    };

    #[test]
    fn test_gid_list_item() {
        let item = GIDListItem::from("https://e-hentai.org/g/2231376/a7584a5932/".to_string());
        assert_eq!(item.0, 2231376);
        assert_eq!(item.1, "a7584a5932".to_string());
    }

    #[test]
    fn test_page_list_item() {
        let item = PageListItem::from("https://e-hentai.org/s/40bc07a79a/618395-11".to_string());
        assert_eq!(item.0, 618395);
        assert_eq!(item.1, "40bc07a79a".to_string());
        assert_eq!(item.2, 11);
    }

    #[tokio::test]
    async fn test_gallery_metadata_request() {
        let proxy = EhClientProxy::new("http", "127.0.0.1", 7890);
        let config = EhClientConfig {
            site: Site::Eh,
            proxy: Some(proxy),
            auth: None,
        };
        let client = EhClient::new(config);
        let url = Url::parse("https://api.e-hentai.org/api.php").unwrap();
        // let body =
        //     GalleryMetadataRequest::new(vec![GIDListItem(2465890, "8af9a35448".to_string())]);
        let body = GalleryMetadataRequest::new(vec![GIDListItem::from(
            "https://e-hentai.org/g/2519745/76939e430f/".to_string(),
        )]);
        let body = serde_json::to_string(&body).unwrap();
        let res: Result<GalleryMetadataResponse, String> = client.post_json(url, body).await;
        let res = res.unwrap();
        assert_eq!(res.gmetadata.len(), 1);
        assert_eq!(res.gmetadata[0].gid, 2519745);
        assert_eq!(res.gmetadata[0].token, "76939e430f".to_string());
    }

    #[tokio::test]
    async fn test_gallery_token_request() {
        let proxy = EhClientProxy::new("http", "127.0.0.1", 7890);
        let config = EhClientConfig {
            site: Site::Eh,
            proxy: Some(proxy),
            auth: None,
        };
        let client = EhClient::new(config);
        let url = Url::parse("https://api.e-hentai.org/api.php").unwrap();
        let body = GalleryTokensRequest::new(vec![PageListItem::from(
            "https://e-hentai.org/s/d384d63ec0/2519745-8".to_string(),
        )]);
        let body = serde_json::to_string(&body).unwrap();
        let res: Result<GalleryTokenResponse, String> = client.post_json(url, body).await;
        let res = res.unwrap();
        assert_eq!(res.tokenlist.len(), 1);
        assert_eq!(res.tokenlist[0].gid, 2519745);
        assert_eq!(res.tokenlist[0].token, "76939e430f".to_string());
    }
}
