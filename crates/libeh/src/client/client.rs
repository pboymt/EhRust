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
//! - **请求基线**：Chrome UA、10s 连接超时、gzip 解压、HTML 请求自动
//!   携带站点根 `Referer`；**超时按操作分级**——页面 30s、api.php 15s
//!   （连接超时为客户端级 10s），未来大文件下载可传更长的每请求超时。
//! - **站点能力**：搜索（[`EhClient::search_parsed`]）、画廊元数据
//!   （[`EhClient::gallery_metadata`]）、图片页
//!   （[`EhClient::gallery_page`]）、种子列表（[`EhClient::torrents`]）、
//!   收藏夹（[`EhClient::favorites`] 等三个方法）与账密登录
//!   （[`EhClient::login`]）。
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
    parse_gallery_page_response, GalleryMetadata, GalleryMetadataError, GalleryMetadataRequest,
    GalleryPageApiRequest, GalleryPageApiResult, GalleryTokenResponse, GalleryTokensRequest,
    GidListItem, PageListItem, TokenListItem, API_URL_EH, API_URL_EX, GDATA_MAX_ITEMS,
};
use crate::dto::favorites::{FavoriteCategories, FavoritesPage};
use crate::dto::search_result::SearchResult;
use crate::dto::torrent::TorrentEntry;
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
/// 页面请求（列表/详情/图片页）的超时。
const TIMEOUT_PAGE: Duration = Duration::from_secs(30);
/// api.php JSON 请求的超时。
const TIMEOUT_API: Duration = Duration::from_secs(15);
/// 论坛账密登录地址（IPB 表单）。
const LOGIN_URL: &str = "https://forums.e-hentai.org/index.php?act=Login&CODE=01";
/// 登录表单的 `Referer`（登录页本身）。
const LOGIN_REFERER: &str = "https://forums.e-hentai.org/index.php?act=Login&CODE=00";
/// 登录表单的 `Origin`。
const LOGIN_ORIGIN: &str = "https://forums.e-hentai.org";

/// E-Hentai/ExHentai HTTP 客户端。
///
/// 通过 [`EhClient::try_new`] 构造；客户端可安全 clone/共享（内部为
/// 连接池化的 [`reqwest::Client`]），建议整个进程复用一个实例。
#[derive(Debug, Clone)]
pub struct EhClient {
    site: Site,
    client: Client,
    /// CookieJar 的本体检索句柄：`cookie_provider` 持有同一份 Arc，
    /// [`EhClient::login`] 成功后经由它把论坛 Cookie 补写到两个站点域。
    jar: std::sync::Arc<Jar>,
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
            .user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36 Edg/121.0.0.0",
            );

        if let Some(proxy) = &config.proxy {
            // 协议校验收敛在 EhClientProxy::validate（reqwest 构建期不校验）
            proxy.validate()?;
            let proxy_url = proxy.to_string();
            let proxy = Proxy::all(&proxy_url)
                .map_err(|e| Error::Config(format!("invalid proxy {proxy_url:?}: {e}")))?;
            builder = builder.proxy(proxy);
        }

        let jar = std::sync::Arc::new(Jar::default());
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
        builder = builder.cookie_provider(jar.clone());

        let client = builder.build().map_err(Error::Http)?;

        Ok(EhClient {
            client,
            site: config.site,
            jar,
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
        let request = self.client.get(url.clone()).timeout(TIMEOUT_PAGE).header(
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
            .timeout(TIMEOUT_API)
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
            .timeout(TIMEOUT_API)
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

    /// 构建一个未发送的原始 GET 请求（供下载器等需要自定义
    /// 超时/头部/流式读取的场景使用）。
    pub fn raw_get(&self, url: Url) -> reqwest::RequestBuilder {
        self.client.get(url)
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
                .timeout(TIMEOUT_API)
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
    /// 网络/状态码失败 → 相应 [`Error`]；站点拒绝（顶层 `error` 字段）
    /// → [`Error::Protocol`]；响应缺少 `tokenlist` → [`Error::Parse`]。
    pub async fn gallery_tokens(
        &self,
        items: Vec<PageListItem>,
    ) -> Result<Vec<TokenListItem>, Error> {
        let body = GalleryTokensRequest::new(items);
        let raw = self
            .client
            .post(self.api_url())
            .timeout(TIMEOUT_API)
            .json(&body)
            .send()
            .await
            .map_err(Error::Http)?;
        let status = raw.status();
        if !status.is_success() {
            return Err(Error::status(status.as_u16(), self.api_url().to_string()));
        }
        let text = raw.text().await.map_err(Error::Http)?;
        // 与 gdata 相同：站点对无效 pagelist 返回顶层 {"error": …}
        let value: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| Error::parse("gtoken response", e.to_string(), snippet(&text, 512)))?;
        if let Some(err) = value.get("error").and_then(serde_json::Value::as_str) {
            return Err(Error::Protocol(err.to_string()));
        }
        let response: GalleryTokenResponse = serde_json::from_value(value)
            .map_err(|e| Error::parse("gtoken response", e.to_string(), snippet(&text, 512)))?;
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

impl EhClient {
    /// 通过 api.php 的 `showpage` 方法取单张图片页数据。
    ///
    /// 与抓取 HTML 图片页（[`GalleryPage::parse`](crate::dto::gallery::page::GalleryPage::parse)）
    /// 相比，API 方式不消耗浏览计数且能一并拿到原图链接；
    /// `showkey` 来自 HTML 页面（[`GalleryPage::show_key`](crate::dto::gallery::page::GalleryPage::show_key)）。
    ///
    /// # Errors
    ///
    /// 网络/状态码失败 → 相应 [`Error`]；站点拒绝（`error` 字段、
    /// sad panda 等）→ [`Error::Protocol`]；响应结构异常 → [`Error::Parse`]。
    pub async fn gallery_page(
        &self,
        gid: i64,
        page: i32,
        imgkey: &str,
        showkey: String,
    ) -> Result<GalleryPageApiResult, Error> {
        let body = GalleryPageApiRequest::new(gid, page, imgkey, showkey);
        let raw = self
            .client
            .post(self.api_url())
            .timeout(TIMEOUT_API)
            .json(&body)
            .send()
            .await
            .map_err(Error::Http)?;
        let status = raw.status();
        if !status.is_success() {
            return Err(Error::status(status.as_u16(), self.api_url().to_string()));
        }
        let text = raw.text().await.map_err(Error::Http)?;
        parse_gallery_page_response(&text)
    }

    /// 抓取并解析画廊种子列表。
    ///
    /// `torrent_url` 来自 [`GalleryDetail::torrent_url`](crate::dto::gallery::detail::GalleryDetail::torrent_url)
    /// （`gallerytorrents.php` 弹窗地址，绝对/相对路径均可；相对路径按
    /// 当前站点根补全）。
    ///
    /// # Errors
    ///
    /// 网络/状态码/协议嗅探失败 → 相应 [`Error`]；页面解析失败 → [`Error::Parse`]。
    pub async fn torrents(&self, torrent_url: &str) -> Result<Vec<TorrentEntry>, Error> {
        let url = if torrent_url.starts_with("http") {
            Url::parse(torrent_url).map_err(|e| Error::Config(format!("bad torrent url: {e}")))?
        } else {
            let mut base = self.site.url()?;
            base.set_path(torrent_url.trim_start_matches('/'));
            base
        };
        let html = self.get_html(url).await?;
        TorrentEntry::parse_page(&html)
    }

    /// 账密登录论坛（IPB），成功后自动把会话 Cookie 补写到两个站点域。
    ///
    /// 站点登录态完全由 Cookie 承载：本方法向论坛提交表单
    /// （`UserName`/`PassWord`/`CookieDate=1`/`temporary_https=off`），
    /// 成功后把响应的 `Set-Cookie`（`ipb_member_id`、`ipb_pass_hash` 等）
    /// 同时写入 e-hentai.org 与 exhentai.org 域，客户端即刻具备认证身份。
    ///
    /// 注意：站点在风控时可能要求验证码或人机校验，此时返回
    /// [`Error::Protocol`]，请改用浏览器登录后手工导入 Cookie。
    ///
    /// # Errors
    ///
    /// - IPB 错误框（密码错误、需要验证码等）→ [`Error::Protocol`]；
    /// - 响应既无欢迎语也无错误框（Cloudflare 拦截页）→ [`Error::Parse`]；
    /// - 网络/状态码失败 → 相应 [`Error`]。
    pub async fn login(&self, username: &str, password: &str) -> Result<String, Error> {
        let url = Url::parse(LOGIN_URL).expect("constant url");
        let response = self
            .client
            .post(url.clone())
            .timeout(TIMEOUT_PAGE)
            .header(REFERER, LOGIN_REFERER)
            .header(reqwest::header::ORIGIN, LOGIN_ORIGIN)
            .form(&[
                ("UserName", username),
                ("PassWord", password),
                ("submit", "Log me in"),
                ("CookieDate", "1"),
                ("temporary_https", "off"),
            ])
            .send()
            .await
            .map_err(Error::Http)?;
        let status = response.status();
        if !status.is_success() {
            return Err(Error::status(status.as_u16(), url.to_string()));
        }

        // 论坛只对论坛域下发会话 Cookie；要同时具备主站/里站身份，
        // 必须把 Set-Cookie 复制写入两个站点域（EhViewer 双域复制行为）
        let set_cookies: Vec<String> = response
            .headers()
            .get_all(reqwest::header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .map(std::string::ToString::to_string)
            .collect();
        let body = response.text().await.map_err(Error::Http)?;
        let display_name = crate::dto::signin::parse_sign_in(&body)?;
        for raw in &set_cookies {
            for domain in AUTH_DOMAINS {
                let target = Site::from_domain(domain).url()?;
                self.jar.add_cookie_str(raw, &target);
            }
        }
        Ok(display_name)
    }

    /// 抓取收藏夹页面（槽位统计 + 画廊列表）。
    ///
    /// `favcat` 为收藏夹编号（0–9）；`None` 表示默认收藏夹。
    /// 需要认证 Cookie，未登录时返回 [`Error::Protocol`]。
    ///
    /// # Errors
    ///
    /// 未登录 → [`Error::Protocol`]；网络/状态码/解析失败 → 相应 [`Error`]。
    pub async fn favorites(&self, favcat: Option<u8>) -> Result<FavoritesPage, Error> {
        let mut url = self.site.url()?;
        url.set_path("favorites.php");
        if let Some(fc) = favcat {
            url.query_pairs_mut().append_pair("favcat", &fc.to_string());
        }
        let html = self.get_html(url).await?;
        // 同一页面解析两次（DOM 级槽位 + 字符串级列表）；页面约 100KB，可接受
        let d = scraper::Html::parse_document(&html);
        let categories = FavoriteCategories::parse(&d)?;
        let result = SearchResult::parse(html)?;
        Ok(FavoritesPage { categories, result })
    }

    /// 把一个画廊加入收藏（或从收藏中删除）。
    ///
    /// - `favcat`：`0..=9` 为目标收藏夹编号；`-1` 表示删除收藏；
    /// - `note`：收藏备注（站点限制 250 字符）。
    ///
    /// 需要认证 Cookie。
    ///
    /// # Errors
    ///
    /// 未登录 → [`Error::Protocol`]；`favcat` 越界 → [`Error::Config`]；
    /// 网络/状态码/协议嗅探失败 → 相应 [`Error`]。
    pub async fn add_favorite(
        &self,
        gid: i64,
        token: &str,
        favcat: i8,
        note: &str,
    ) -> Result<(), Error> {
        let cat = match favcat {
            -1 => "favdel".to_string(),
            c if (0..=9).contains(&c) => c.to_string(),
            other => {
                return Err(Error::Config(format!(
                    "invalid favcat {other}; expected -1 (delete) or 0..=9"
                )))
            }
        };
        let mut url = self.site.url()?;
        url.set_path("gallerypopups.php");
        url.query_pairs_mut()
            .append_pair("gid", &gid.to_string())
            .append_pair("t", token)
            .append_pair("act", "addfav");
        self.post_form(
            url,
            &[
                ("favcat", cat),
                ("favnote", note.to_string()),
                ("submit", "Apply Changes".to_string()),
                ("update", "1".to_string()),
            ],
        )
        .await?;
        Ok(())
    }

    /// 批量移动/删除收藏（收藏夹页面勾选后 Apply 的表单等价物）。
    ///
    /// - `gids`：要操作的画廊 ID 列表；
    /// - `dst_cat`：`0..=9` 为目标收藏夹编号；`-1` 表示删除收藏。
    ///
    /// 需要认证 Cookie。
    ///
    /// # Errors
    ///
    /// 未登录 → [`Error::Protocol`]；`dst_cat` 越界 → [`Error::Config`]；
    /// `gids` 为空 → [`Error::Config`]；网络/状态码失败 → 相应 [`Error`]。
    pub async fn modify_favorites(&self, gids: &[i64], dst_cat: i8) -> Result<(), Error> {
        if gids.is_empty() {
            return Err(Error::Config("gids is empty".into()));
        }
        let ddact = match dst_cat {
            -1 => "delete".to_string(),
            c if (0..=9).contains(&c) => format!("fav{c}"),
            other => {
                return Err(Error::Config(format!(
                    "invalid dst_cat {other}; expected -1 (delete) or 0..=9"
                )))
            }
        };
        let mut url = self.site.url()?;
        url.set_path("favorites.php");
        let mut form: Vec<(String, String)> = vec![
            ("ddact".to_string(), ddact),
            ("apply".to_string(), "Apply".to_string()),
        ];
        for gid in gids {
            form.push(("modifygids[]".to_string(), gid.to_string()));
        }
        let form_refs: Vec<(&str, String)> =
            form.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
        self.post_form(url, &form_refs).await?;
        Ok(())
    }

    /// POST 一个 urlencoded 表单并返回响应文本（带状态码检查与协议嗅探）。
    ///
    /// 收藏夹等表单操作的公共底座；`Referer`/`Origin` 设为站点根。
    async fn post_form(&self, url: Url, form: &[(&str, String)]) -> Result<String, Error> {
        let origin = format!("https://{}/", url.host_str().unwrap_or_default());
        let response = self
            .client
            .post(url.clone())
            .timeout(TIMEOUT_PAGE)
            .header(REFERER, origin.clone())
            .header(reqwest::header::ORIGIN, origin)
            .form(form)
            .send()
            .await
            .map_err(Error::Http)?;
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
        if crate::utils::regex::contains_phrase(&text, "This page requires you to log on.") {
            return Err(Error::Protocol("This page requires you to log on.".into()));
        }
        Ok(text)
    }
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
