//! E-Hentai/ExHentai HTTP 客户端。
//!
//! [`EhClient`] 封装 [`reqwest::Client`]，负责：
//!
//! - **认证注入**：把 [`EhClientAuth`](crate::client::auth::EhClientAuth) 的三个 Cookie（`ipb_member_id`、
//!   `ipb_pass_hash`、`igneous`）**同时写入 e-hentai.org 与 exhentai.org
//!   两个域**（与 EhViewer 的双域复制行为一致），并额外注入 `nw=1`
//!   跳过表站的成人内容警告页；
//! - **错误分类**：非 2xx → [`Error::Status`]；HTTP 200 但内容表示拒绝
//!   （sad panda、kokomade 等）→ [`Error::Protocol`]；
//! - **请求基线**：Chrome UA、10s 连接超时 / 30s 总超时、gzip 解压、
//!   HTML 请求自动携带站点根 `Referer`。
//!
//! # 示例
//!
//! ```no_run
//! use libeh::client::{client::EhClient, config::EhClientConfig};
//! use libeh::dto::keyword::Keyword;
//!
//! # async fn demo() -> Result<(), libeh::error::Error> {
//! let config = EhClientConfig {
//!     auth: Some(libeh::client::auth::EhClientAuth::new("1234", "hash", Some("igneous"))),
//!     ..EhClientConfig::default()
//! };
//! let client = EhClient::try_new(config)?;
//! let result = client
//!     .search_parsed(vec![Keyword::Artist("simon".into())], None)
//!     .await?;
//! println!("pages: {}", result.pages);
//! # Ok(())
//! # }
//! ```

use std::time::Duration;

use reqwest::header::{CONTENT_DISPOSITION, REFERER};
use reqwest::redirect;
use reqwest::{cookie::Jar, Client, Proxy, Url};
use serde::de::DeserializeOwned;

use crate::dto::api::{
    GalleryMetadata, GalleryMetadataError, GalleryMetadataRequest, GalleryTokenResponse,
    GalleryTokensRequest, GidListItem, PageListItem, TokenListItem, API_URL_EH, API_URL_EX,
    GDATA_MAX_ITEMS,
};
use crate::dto::search_result::SearchResult;
use crate::dto::{keyword::Keyword, search_offset::Offset, site::Site};
use crate::error::{snippet, Error};
use crate::url::search::SearchBuilder;

use super::config::EhClientConfig;

/// 未登录访问 ExHentai 时站点返回的 sad panda 图片的响应头特征。
const SAD_PANDA_DISPOSITION: &str = "sadpanda.jpg";
/// ExHentai 引导页（kokomade）的特征文本。
const KOKOMADE_MARK: &str = "exhentai.org/img/kokomade.jpg";
/// 画廊不可见页的特征文本。
const UNAVAILABLE_MARK: &str = "This gallery is unavailable";
/// 表站成人内容警告页的跳过 Cookie（`nw=1`），EhViewer 亦强制注入。
const CONTENT_WARNING_COOKIE: (&str, &str) = ("nw", "1");
/// 认证 Cookie 需要同时写入的两个站点域（与 EhViewer 行为一致）。
const AUTH_DOMAINS: [&str; 2] = ["e-hentai.org", "exhentai.org"];
/// 客户端支持的代理协议（`socks5` 依赖 reqwest 的 `socks` feature）。
const SUPPORTED_PROXY_PROTOCOLS: [&str; 3] = ["http", "https", "socks5"];

/// E-Hentai/ExHentai HTTP 客户端。
///
/// 通过 [`EhClient::try_new`] 构造；客户端可安全 clone/共享（内部为
/// 连接池化的 [`reqwest::Client`]），建议整个进程复用一个实例。
#[derive(Debug, Clone)]
pub struct EhClient {
    site: Site,
    client: Client,
}

impl EhClient {
    /// 从配置构建客户端。
    ///
    /// 认证 Cookie（如配置了 [`EhClientAuth`](crate::client::auth::EhClientAuth)）
    /// 会写入 `AUTH_DOMAINS` 两个域，并附加 `nw=1`；
    /// 代理配置非法时返回错误而不是静默忽略。
    ///
    /// # Errors
    ///
    /// - 配置的代理无法解析或协议不受支持 → [`Error::Config`]；
    /// - [`reqwest::Client`] 构建失败（TLS 后端初始化等）→ [`Error::Http`]。
    pub fn try_new(config: EhClientConfig) -> Result<Self, Error> {
        let mut builder = Client::builder()
            .redirect(redirect::Policy::limited(20))
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36 Edg/121.0.0.0",
            );

        if let Some(proxy) = &config.proxy {
            let proxy_url = proxy.to_string();
            // reqwest 构建期不校验协议，这里先行校验以给出明确错误
            if !SUPPORTED_PROXY_PROTOCOLS.contains(&proxy.protocol.as_str()) {
                return Err(Error::Config(format!(
                    "unsupported proxy protocol {:?}; expected one of {SUPPORTED_PROXY_PROTOCOLS:?}",
                    proxy.protocol
                )));
            }
            let proxy = Proxy::all(&proxy_url)
                .map_err(|e| Error::Config(format!("invalid proxy {proxy_url:?}: {e}")))?;
            builder = builder.proxy(proxy);
        }

        let jar = Jar::default();
        // reqwest 的 CookieJar 以 Set-Cookie 字符串形态接收条目，
        // 借助 cookie crate 构造带域/路径属性的标准字符串；
        // 认证 Cookie 与 nw=1 同时写入两个站点域（与 EhViewer 行为一致）
        let mut cookies: Vec<(String, &'static str)> = Vec::new();
        for domain in AUTH_DOMAINS {
            if let Some(auth) = &config.auth {
                for (key, value) in auth.to_pairs() {
                    cookies.push((
                        cookie::Cookie::build((key.as_str(), value.as_str()))
                            .domain(domain)
                            .path("/")
                            .build()
                            .to_string(),
                        domain,
                    ));
                }
            }
            cookies.push((
                cookie::Cookie::build((CONTENT_WARNING_COOKIE.0, CONTENT_WARNING_COOKIE.1))
                    .domain(domain)
                    .path("/")
                    .build()
                    .to_string(),
                domain,
            ));
        }
        for (raw, domain) in cookies {
            let url = Site::from_domain(domain).url()?;
            jar.add_cookie_str(&raw, &url);
        }
        builder = builder.cookie_provider(jar.into());

        let client = builder.build().map_err(Error::Http)?;

        Ok(EhClient {
            client,
            site: config.site,
        })
    }

    /// [`EhClient::try_new`] 的 panic 版本：构建失败时直接 panic。
    ///
    /// 适用于配置来自受控来源（测试、示例）的场景；
    /// 面向用户的程序请使用 [`EhClient::try_new`]。
    ///
    /// # Panics
    ///
    /// 代理配置非法或 [`reqwest::Client`] 构建失败时 panic。
    pub fn new(config: EhClientConfig) -> Self {
        EhClient::try_new(config).expect("failed to build EhClient")
    }

    /// 当前客户端绑定的站点。
    #[must_use]
    pub fn site(&self) -> Site {
        self.site
    }

    /// 搜索并返回原始 HTML（[`SearchBuilder`] 的快捷方式）。
    ///
    /// 需要解析后的结构化结果时请使用 [`EhClient::search_parsed`]。
    ///
    /// # Errors
    ///
    /// 搜索 URL 构建失败（[`Site::Un`]）、网络失败、状态码非 2xx
    /// 或站点返回协议级错误时返回相应 [`Error`]。
    pub async fn search(
        &self,
        keywords: Vec<Keyword>,
        offset: Option<Offset>,
    ) -> Result<String, Error> {
        let mut builder = SearchBuilder::new(self.site).add_keywords(keywords);
        if let Some(offset) = offset {
            builder = builder.offset(offset);
        }
        let url = builder.build()?;
        self.get_html(url).await
    }

    /// 搜索并解析为 [`SearchResult`]（[`EhClient::search`] 的结构化版本）。
    ///
    /// # Errors
    ///
    /// 除 [`EhClient::search`] 的错误外，页面解析失败返回 [`Error::Parse`]。
    pub async fn search_parsed(
        &self,
        keywords: Vec<Keyword>,
        offset: Option<Offset>,
    ) -> Result<SearchResult, Error> {
        let html = self.search(keywords, offset).await?;
        SearchResult::parse(html)
    }

    /// GET 一个页面并返回响应文本。
    ///
    /// 自动携带站点根 `Referer`；在读取响应体前后分别进行
    /// 状态码检查与协议级错误嗅探（sad panda / kokomade / 画廊不可见）。
    ///
    /// # Errors
    ///
    /// - 网络失败 → [`Error::Http`]；
    /// - 非 2xx → [`Error::Status`]；
    /// - HTTP 200 但站点拒绝 → [`Error::Protocol`]。
    pub async fn get_html(&self, url: Url) -> Result<String, Error> {
        let request = self.client.get(url.clone()).header(
            REFERER,
            format!("https://{}/", url.host_str().unwrap_or_default()),
        );
        let response = request.send().await.map_err(Error::Http)?;
        let status = response.status();
        if !status.is_success() {
            return Err(Error::status(status.as_u16(), url.to_string()));
        }
        let disposition = response
            .headers()
            .get(CONTENT_DISPOSITION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let text = response.text().await.map_err(Error::Http)?;
        if disposition.contains(SAD_PANDA_DISPOSITION) {
            return Err(Error::Protocol("sad panda".into()));
        }
        if text.contains(KOKOMADE_MARK) {
            return Err(Error::Protocol("kokomade".into()));
        }
        if text.contains(UNAVAILABLE_MARK) {
            return Err(Error::Protocol("gallery unavailable".into()));
        }
        Ok(text)
    }

    /// GET 一个 URL 并把响应反序列化为 JSON。
    ///
    /// # Errors
    ///
    /// 网络失败 → [`Error::Http`]；非 2xx → [`Error::Status`]；
    /// 响应体不是合法 JSON 或与 `T` 不匹配 → [`Error::Parse`]。
    pub async fn get_json<T>(&self, url: Url) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .get(url.clone())
            .send()
            .await
            .map_err(Error::Http)?;
        let status = response.status();
        if !status.is_success() {
            return Err(Error::status(status.as_u16(), url.to_string()));
        }
        response
            .json::<T>()
            .await
            .map_err(|e| Error::parse("json response", e.to_string(), String::new()))
    }

    /// POST 一个 JSON 请求体（自动设置 `Content-Type: application/json`）
    /// 并把响应反序列化为 JSON。
    ///
    /// # Errors
    ///
    /// 请求体序列化失败、网络失败、非 2xx、响应解析失败时返回相应 [`Error`]。
    pub async fn post_json<B, R>(&self, url: Url, body: &B) -> Result<R, Error>
    where
        B: serde::Serialize,
        R: DeserializeOwned,
    {
        let response = self
            .client
            .post(url.clone())
            .json(body)
            .send()
            .await
            .map_err(Error::Http)?;
        let status = response.status();
        if !status.is_success() {
            return Err(Error::status(status.as_u16(), url.to_string()));
        }
        response
            .json::<R>()
            .await
            .map_err(|e| Error::parse("json response", e.to_string(), String::new()))
    }

    /// 当前站点对应的 api.php 地址。
    #[must_use]
    pub fn api_url(&self) -> Url {
        match self.site {
            Site::Ex => Url::parse(API_URL_EX).expect("constant url"),
            _ => Url::parse(API_URL_EH).expect("constant url"),
        }
    }

    /// 通过 api.php 的 `gdata` 方法批量获取画廊元数据。
    ///
    /// 站点限制单次请求最多 [`GDATA_MAX_ITEMS`]（25）条；
    /// 本方法自动分批串行请求并拼接结果。
    ///
    /// # Errors
    ///
    /// - 任一批次整体报错（api.php 顶层 `error` 字段）→ [`Error::Protocol`]；
    /// - 网络/状态码/解析失败 → 相应 [`Error`]。
    ///   单个画廊条目内的错误以 [`GidDataResult::Error`] 形式保留在返回值中，
    ///   不会中断其余条目。
    pub async fn gallery_metadata(
        &self,
        items: Vec<GidListItem>,
    ) -> Result<Vec<GidDataResult>, Error> {
        let mut results = Vec::with_capacity(items.len());
        for chunk in items.chunks(GDATA_MAX_ITEMS) {
            let body = GalleryMetadataRequest::new(chunk.to_vec())?;
            let raw = self
                .client
                .post(self.api_url())
                .json(&body)
                .send()
                .await
                .map_err(Error::Http)?;
            let status = raw.status();
            if !status.is_success() {
                return Err(Error::status(status.as_u16(), self.api_url().to_string()));
            }
            let text = raw.text().await.map_err(Error::Http)?;
            let value: serde_json::Value = serde_json::from_str(&text)
                .map_err(|e| Error::parse("gdata response", e.to_string(), snippet(&text, 512)))?;
            if let Some(err) = value.get("error").and_then(|v| v.as_str()) {
                return Err(Error::Protocol(err.to_string()));
            }
            let Some(entries) = value.get("gmetadata").and_then(|v| v.as_array()) else {
                return Err(Error::parse(
                    "gdata response",
                    "missing gmetadata array",
                    snippet(&text, 512),
                ));
            };
            for entry in entries {
                match serde_json::from_value::<GalleryMetadata>(entry.clone()) {
                    Ok(metadata) => results.push(GidDataResult::Metadata(Box::new(metadata))),
                    Err(_) => {
                        if let Ok(err) =
                            serde_json::from_value::<GalleryMetadataError>(entry.clone())
                        {
                            results.push(GidDataResult::Error(err));
                        } else {
                            results.push(GidDataResult::Error(GalleryMetadataError {
                                gid: entry.get("gid").and_then(|v| v.as_i64()).unwrap_or(-1),
                                error: "unrecognized gmetadata entry".to_string(),
                            }));
                        }
                    }
                }
            }
        }
        Ok(results)
    }

    /// 通过 api.php 的 `gtoken` 方法由页面 URL 反查画廊令牌。
    ///
    /// # Errors
    ///
    /// 网络/状态码失败 → 相应 [`Error`]；响应缺少 `tokenlist` → [`Error::Parse`]。
    pub async fn gallery_tokens(
        &self,
        items: Vec<PageListItem>,
    ) -> Result<Vec<TokenListItem>, Error> {
        let body = GalleryTokensRequest::new(items);
        let response: GalleryTokenResponse = self.post_json(self.api_url(), &body).await?;
        Ok(response.tokenlist)
    }
}

/// `gdata` 响应中单个画廊条目的结果。
///
/// api.php 对无效的 gid/token 会在 `gmetadata` 数组内返回错误条目，
/// 此时该条目以 [`GidDataResult::Error`] 保留，不影响同批其余条目。
#[derive(Debug, Clone)]
pub enum GidDataResult {
    /// 画廊元数据（Box 装箱以缩小枚举尺寸）。
    Metadata(Box<GalleryMetadata>),
    /// 该条目查询失败（`error` 字段原文）。
    Error(GalleryMetadataError),
}

#[cfg(test)]
mod tests {
    use crate::client::config::EhClientConfig;
    use crate::client::proxy::EhClientProxy;
    use crate::dto::site::Site;

    use super::EhClient;

    #[tokio::test]
    async fn builds_without_network() {
        // 构建客户端不需要网络：Cookie 注入、代理解析、TLS 初始化全部离线完成
        let config = EhClientConfig {
            site: Site::Eh,
            proxy: Some(EhClientProxy::new("http", "127.0.0.1", 7890)),
            auth: Some(crate::client::auth::EhClientAuth::new("1", "hash", None)),
        };
        let client = EhClient::try_new(config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn invalid_proxy_is_config_error() {
        let config = EhClientConfig {
            site: Site::Eh,
            proxy: Some(EhClientProxy::new("gopher", "127.0.0.1", 70)),
            auth: None,
        };
        let err = match EhClient::try_new(config) {
            Err(e) => e,
            Ok(_) => panic!("expected config error"),
        };
        assert!(matches!(err, crate::error::Error::Config(_)));
    }
}
