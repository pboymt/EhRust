//! 画廊图片批量下载器。
//!
//! 把"画廊 URL → 全部原图落盘"的完整流水线封装为一个调用：
//!
//! ```text
//! GalleryBuilder
//!   │ 1. 详情页：总页数 + 预览分页数（GalleryDetail / GalleryPreview）
//!   │ 2. 遍历预览页 g/{gid}/{token}/?p=0..total_set：收集全部
//!   │    /s/{pToken}/{gid}-{page} 链接（GalleryPreviewPage.link）
//!   │ 3. 并发抓取图片页 /s/…（GalleryPage::parse → image_url）
//!   │ 4. 并发下载图片（Referer=/s/…，Range 断点续传，.part 临时文件）
//!   ▼
//! dest/{gid}/{page:04}.{ext}
//! ```
//!
//! # 下载语义
//!
//! - **并发**：[`DownloadOptions::concurrency`] 限制同时进行的图片页
//!   抓取与图片下载（站点对高并发敏感，默认 3）；
//! - **断点续传**：写入先落 `.{page}.part`，已完成的部分以
//!   `Range: bytes={len}-` 续传（206 追加 / 200 重写），全部写完后
//!   原子改名去掉 `.part`；
//! - **跳过**：目标文件已存在且 [`DownloadOptions::overwrite`] 为
//!   `false` 时跳过（对齐"下载目录去重"的习惯）；
//! - **进度**：[`ProgressEvent`] 通过回调实时上报，构造 UI 进度条
//!   或 CLI 输出。
//!
//! 单页失败不会中断整体——失败页记录在 [`DownloadSummary::failed`]。
//!
//! # 示例
//!
//! ```no_run
//! use std::path::PathBuf;
//! use libeh::client::{client::EhClient, config::EhClientConfig, downloader::{DownloadOptions, Downloader}};
//! use libeh::url::gallery::GalleryBuilder;
//!
//! # async fn demo() -> Result<(), libeh::error::Error> {
//! let client = EhClient::try_new(EhClientConfig::env()?)?;
//! let gallery = GalleryBuilder::parse("https://e-hentai.org/g/2791585/3e7e1c7107/".into())?;
//!
//! let summary = Downloader::new(client)
//!     .download_gallery(
//!         &gallery,
//!         PathBuf::from("~/pics"),
//!         DownloadOptions::default(),
//!         Some(std::sync::Arc::new(|event| println!("{event:?}"))),
//!     )
//!     .await?;
//! println!("完成 {} 跳过 {} 失败 {}", summary.completed, summary.skipped, summary.failed.len());
//! # Ok(())
//! # }
//! ```

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;

use crate::client::client::EhClient;
use crate::dto::api::PageListItem;
use crate::dto::gallery::detail::GalleryDetail;
use crate::dto::gallery::page::GalleryPage;
use crate::error::Error;
use crate::url::gallery::GalleryBuilder;

/// 单个图片请求的超时（大图 + 慢速 H@H 源，取宽裕值；复审 N1 的分级超时）。
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);
/// 断点续传探测不适用时（服务器忽略 Range 返回 200）的响应码。
const HTTP_OK: u16 = 200;
const HTTP_PARTIAL: u16 = 206;

/// 下载配置。
#[derive(Debug, Clone)]
pub struct DownloadOptions {
    /// 同时进行的图片页抓取/图片下载数（站点敏感，默认 3）。
    pub concurrency: usize,
    /// 目标文件已存在时是否重新下载（默认跳过）。
    pub overwrite: bool,
    /// 单个图片请求的超时（默认 300s）。
    pub request_timeout: Duration,
}

impl Default for DownloadOptions {
    /// 默认：并发 3、不覆盖已存在文件、单图 300s 超时。
    fn default() -> Self {
        DownloadOptions {
            concurrency: 3,
            overwrite: false,
            request_timeout: DOWNLOAD_TIMEOUT,
        }
    }
}

/// 下载进度事件。
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    /// 计划阶段完成：共发现 `total` 张图片。
    Planned(usize),
    /// 一张图片开始下载（续传时 `resumed_bytes` 为已有部分的大小）。
    PageStarted(i32, u64),
    /// 一张图片完成（`bytes` 为本次写入的字节数）。
    PageFinished(i32, u64),
    /// 一张图片被跳过（目标文件已存在）。
    PageSkipped(i32),
    /// 一张图片失败。
    PageFailed(i32, String),
}

/// 下载结果汇总。
#[derive(Debug, Clone, Default)]
pub struct DownloadSummary {
    /// 成功写入的图片数。
    pub completed: usize,
    /// 因目标文件已存在而跳过的数量。
    pub skipped: usize,
    /// 失败的页号与原因（不中断整体）。
    pub failed: Vec<(i32, String)>,
}

/// 画廊图片下载器。
///
/// 封装 [`EhClient`] 与下载策略；结构本身无状态，
/// [`Downloader::download_gallery`] 每次调用完整走一遍流水线。
#[derive(Debug, Clone)]
pub struct Downloader {
    client: EhClient,
}

impl Downloader {
    /// 以客户端创建下载器。
    #[must_use]
    pub fn new(client: EhClient) -> Self {
        Downloader { client }
    }

    /// 下载整个画廊到 `dest`（自动创建 `dest/{gid}/` 目录）。
    ///
    /// 流程见[模块文档](self)。`progress` 传 `None` 可关闭进度上报。
    ///
    /// # Errors
    ///
    /// 计划阶段（详情页/预览页抓取）失败 → 相应 [`Error`]；
    /// 单张图片的失败不产生 `Err`，记入 [`DownloadSummary::failed`]。
    pub async fn download_gallery(
        &self,
        gallery: &GalleryBuilder,
        dest: &Path,
        options: DownloadOptions,
        progress: Option<ProgressCallback>,
    ) -> Result<DownloadSummary, Error> {
        // 1. 详情页：总页数与预览分页数
        let detail_url = gallery.url();
        let detail = GalleryDetail::parse(self.client.get_html(detail_url).await?)?;
        let total_pages = detail.info.pages.max(0) as i32;
        let total_sets = detail.preview.total_set.max(1);
        let base_url = gallery.url();

        // 2. 遍历预览分页收集 (page, pToken)
        //    预览分页可能比总页数先耗尽（站点上限），两种终止条件都处理
        let mut tokens: Vec<PageListItem> = Vec::with_capacity(total_pages as usize);
        for preview_page in 0..total_sets {
            let mut url = base_url.clone();
            url.set_query(Some(&format!("p={preview_page}")));
            let html = self.client.get_html(url).await?;
            let d = scraper::Html::parse_document(&html);
            let pages = crate::dto::gallery::preview::GalleryPreview::parse_preview_pages(&d)?;
            for item in pages {
                if let Ok(list_item) = PageListItem::try_from(item.link.clone()) {
                    tokens.push(list_item);
                }
            }
            if tokens.len() >= total_pages as usize {
                break;
            }
        }
        tokens.truncate(total_pages as usize);
        let total = tokens.len();
        if let Some(cb) = &progress {
            cb(ProgressEvent::Planned(total));
        }

        // 3/4. 并发下载
        let out_dir = dest.join(gallery.gid.to_string());
        tokio::fs::create_dir_all(&out_dir)
            .await
            .map_err(|e| Error::Config(format!("cannot create {}: {e}", out_dir.display())))?;

        let semaphore = Arc::new(Semaphore::new(options.concurrency.max(1)));
        let mut handles = Vec::with_capacity(total);
        for item in tokens {
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| Error::Config(format!("downloader semaphore closed: {e}")))?;
            let client = self.client.clone();
            let out_dir = out_dir.clone();
            let base_url = base_url.clone();
            let options = options.clone();
            let progress = progress.clone();
            handles.push(tokio::spawn(async move {
                let (gid, ptoken, page) = (item.0, item.1.clone(), item.2);
                let _permit = permit;
                let result =
                    download_one(&client, &base_url, gid, &ptoken, page, &out_dir, &options).await;
                report(&progress, page, &result);
                (page, result)
            }));
        }

        let mut summary = DownloadSummary::default();
        for handle in handles {
            let (page, result) = handle
                .await
                .map_err(|e| Error::Config(format!("download task panicked: {e}")))?;
            match result {
                Ok(Outcome::Completed(bytes)) => {
                    summary.completed += 1;
                    if let Some(cb) = &progress {
                        cb(ProgressEvent::PageFinished(page, bytes));
                    }
                }
                Ok(Outcome::Skipped) => {
                    summary.skipped += 1;
                    if let Some(cb) = &progress {
                        cb(ProgressEvent::PageSkipped(page));
                    }
                }
                Err(e) => {
                    if let Some(cb) = &progress {
                        cb(ProgressEvent::PageFailed(page, e.to_string()));
                    }
                    summary.failed.push((page, e.to_string()));
                }
            }
        }
        Ok(summary)
    }
}

/// 进度回调类型（跨线程共享）。
pub type ProgressCallback = Arc<dyn Fn(ProgressEvent) + Send + Sync>;

/// 把任务结果转发给进度回调（完成事件在汇总处上报，此处只报跳过/失败）。
fn report(progress: &Option<ProgressCallback>, page: i32, result: &Result<Outcome, Error>) {
    if let Some(cb) = progress {
        match result {
            Ok(Outcome::Completed(_)) => {}
            Ok(Outcome::Skipped) => cb(ProgressEvent::PageSkipped(page)),
            Err(e) => cb(ProgressEvent::PageFailed(page, e.to_string())),
        }
    }
}

/// 单页下载的内部结果。
enum Outcome {
    /// 完成并写入 `bytes` 字节。
    Completed(u64),
    /// 目标文件已存在，跳过。
    Skipped,
}

/// 下载单张图片：图片页 HTML → 图片地址 → 落盘（.part + Range 续传）。
async fn download_one(
    client: &EhClient,
    base_url: &reqwest::Url,
    gid: i64,
    ptoken: &str,
    page: i32,
    out_dir: &Path,
    options: &DownloadOptions,
) -> Result<Outcome, Error> {
    // 目标文件：dest/{gid}/{page:04}.{ext}；扩展名下载时才能确定，
    // 已存在的最终文件按常见扩展名匹配以支持"跳过"
    let stem = format!("{page:04}");
    if !options.overwrite {
        for ext in ["jpg", "png", "gif", "webp"] {
            if out_dir.join(format!("{stem}.{ext}")).exists() {
                return Ok(Outcome::Skipped);
            }
        }
    }

    // 1. 图片页 HTML（Referer = 画廊详情页）
    let mut page_url = base_url.clone();
    page_url.set_path(&format!("s/{ptoken}/{gid}-{page}"));
    let html = client.get_html(page_url.clone()).await?;
    let page_info = GalleryPage::parse(&html)?;

    // 2. 图片字节流（Referer = 图片页，站点以此校验防盗链）
    let image_url = page_info
        .image_url
        .parse::<reqwest::Url>()
        .map_err(|e| Error::parse("image url", e.to_string(), page_info.image_url.clone()))?;
    let part_path = out_dir.join(format!("{stem}.part"));
    let resume_from = tokio::fs::metadata(&part_path).await.ok().map(|m| m.len());

    let mut request = client
        .raw_get(image_url.clone())
        .timeout(options.request_timeout)
        .header(reqwest::header::REFERER, page_url.as_str());
    if let Some(len) = resume_from {
        request = request.header(reqwest::header::RANGE, format!("bytes={len}-"));
    }
    let response = request.send().await.map_err(Error::Http)?;
    let status = response.status().as_u16();
    if status != HTTP_OK && status != HTTP_PARTIAL {
        return Err(Error::status(status, image_url.to_string()));
    }
    // 200 = 服务器忽略 Range，从头重写；206 = 续传追加
    let append = status == HTTP_PARTIAL && resume_from.is_some();
    let mut written: u64 = if append { resume_from.unwrap_or(0) } else { 0 };

    let mut file = if append {
        tokio::fs::OpenOptions::new()
            .append(true)
            .open(&part_path)
            .await
            .map_err(|e| Error::Config(format!("open {}: {e}", part_path.display())))?
    } else {
        tokio::fs::File::create(&part_path)
            .await
            .map_err(|e| Error::Config(format!("create {}: {e}", part_path.display())))?
    };

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;
    let mut buffer = tokio::io::BufWriter::new(&mut file);
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(Error::Http)?;
        written += chunk.len() as u64;
        buffer
            .write_all(&chunk)
            .await
            .map_err(|e| Error::Config(format!("write part: {e}")))?;
    }
    buffer
        .flush()
        .await
        .map_err(|e| Error::Config(format!("flush part: {e}")))?;
    drop(buffer);
    file.sync_all()
        .await
        .map_err(|e| Error::Config(format!("sync part: {e}")))?;

    // 3. 按内容类型定最终扩展名并原子改名
    let ext = extension_for(&content_type, &page_info.image_url);
    let final_path = out_dir.join(format!("{stem}.{ext}"));
    tokio::fs::rename(&part_path, &final_path)
        .await
        .map_err(|e| Error::Config(format!("rename to {}: {e}", final_path.display())))?;
    Ok(Outcome::Completed(written))
}

/// 按 `Content-Type` 与 URL 后缀推断文件扩展名（不含点）。
fn extension_for(content_type: &str, image_url: &str) -> &'static str {
    let ct = content_type.split(';').next().unwrap_or("").trim();
    match ct {
        "image/jpeg" | "image/jpg" => return "jpg",
        "image/png" => return "png",
        "image/gif" => return "gif",
        "image/webp" => return "webp",
        _ => {}
    }
    let lower = image_url.to_ascii_lowercase();
    for (needle, ext) in [
        (".jpg", "jpg"),
        (".jpeg", "jpg"),
        (".png", "png"),
        (".gif", "gif"),
        (".webp", "webp"),
    ] {
        if lower.contains(needle) {
            return ext;
        }
    }
    "jpg"
}

#[cfg(test)]
mod tests {
    use super::{extension_for, DownloadOptions};

    #[test]
    fn extension_prefers_content_type() {
        assert_eq!(
            extension_for("image/png; charset=binary", "http://x/a"),
            "png"
        );
        assert_eq!(extension_for("image/jpeg", "http://x/a"), "jpg");
        // 无 Content-Type 时回退 URL 后缀
        assert_eq!(extension_for("", "http://x/a/b/1.PNG?nl=1"), "png");
        assert_eq!(extension_for("", "http://x/a.gif"), "gif");
        // 都没有时默认 jpg
        assert_eq!(extension_for("", "http://x/a"), "jpg");
    }

    #[test]
    fn defaults_are_conservative() {
        let options = DownloadOptions::default();
        assert_eq!(options.concurrency, 3);
        assert!(!options.overwrite);
        assert_eq!(options.request_timeout.as_secs(), 300);
    }
}
