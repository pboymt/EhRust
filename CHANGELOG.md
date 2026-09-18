# Changelog

## 0.2.0（未发布）

本次为一次集中修复与工程化改造，基于对协议实现与代码质量的全面审查
（问题清单与修复计划见 Ehviewer 项目文档库）。

### 协议修复

- **关键词解析**：`Keyword::FromStr` 改为按第一个冒号切分命名空间，
  值允许再含冒号（`parody:re:zero …` 不再导致整页解析失败）；
  未知命名空间回退为 `Normal`（解析永不失败，`Err = Infallible`）；
  值中的 `"` 在生成查询时剔除；`Normal` 不再加引号；`Uploader` 去除 `$` 后缀。
- **高级搜索**：最低评分补发 `f_sr=on`（缺失时站点忽略 `f_srdd`）；
  页数范围补发 `f_sp=on`；`rating()` 收紧为合法值域 `2..=5`。
- **分类掩码**：恢复 `Category::AsianPorn` 位；`All/Private/Unknown`
  不再产出非法 `f_cats` 值；新增语义化 API
  `disable_category`/`enable_category`/`disable_all_categories`/`enable_all_categories`。
- **搜索分页**：`SearchResult` 同时支持 `table.ptt` 数字分页与
  `div.searchnav` 快速分页；`pages`/`next_page`/`result_count`/`no_watched_tags`
  全部正确填充；新增 `skipped_rows` 暴露静默跳过的行。
- **Cookie 策略**：认证 Cookie 同时写入 e-hentai.org 与 exhentai.org 两域；
  附加 `nw=1` 跳过表站内容警告页；Cookie 由 `cookie` crate 构造（带域/路径属性）。
- **错误识别**：非 2xx → `Error::Status`；sad panda / kokomade / 画廊不可用 /
  Offensive / Pining / api.php `error` 字段 → `Error::Protocol`；
  `SearchResult::parse`、`GalleryDetail::parse` 等全面迁移到统一错误类型。
- **api.php**：新增 `API_URL_EH/EX` 常量与 `GDATA_MAX_ITEMS=25` 上限校验；
  `EhClient::gallery_metadata` 自动分批，条目级错误以 `GidDataResult::Error` 保留；
  `post_json` 使用 `.json()` 自动设置 `Content-Type: application/json`。

### 解析器

- 画廊详情：`GalleryPreview::parse` 正式接入 `GalleryDetail::parse`
  （此前从未被调用，preview 恒为空）；预览解析兼容新版（gt200/gt100）与
  旧版（gdtm 雪碧图）两种版式；Torrent/Archive 链接改为文本锚定提取，
  不再依赖 `#gd5` 内条目顺序；`PATTERN_DETAIL` 放宽为 `[\s\S]*?`；
  `rating_label` 显式处理 "Not Yet Rated"。
- 列表页：行解析支持 Minimal/MinimalPlus/Compact/Extended/Thumbnail 五种版式
  （缩略图按 `glthumb → gl1e → gl3t` 回退，上传者按 `/uploader/` 锚点提取）。
- 评论：补齐 `.c4` 操作位解析（`Vote+`/`Vote-`/`Edit` 与已投状态）；
  投票明细解析失败不再中断全部评论。
- `GalleryInfo` 新增 `posted_raw` 保留页面渲染时间原文（时区说明见文档）；
  收藏槽位未知色值与 EhViewer 对齐为 `-2`。
- serde 辅助反序列化不再 `unwrap`（损坏数据返回 serde 错误而非 panic），
  并支持千分位逗号。
- 特殊页嗅探改用"忽略空白差异"的短语匹配（站点文案常在词间换行）。

### 客户端

- `EhClient::try_new`（返回 `Result`）；`new` 保留为 panic 版本并文档化。
- 请求基线：10s 连接超时 / 30s 总超时、gzip 解压、HTML 请求带站点 `Referer`。
- 代理：协议白名单校验（非法配置返回 `Error::Config` 而非静默忽略）；
  启用 reqwest `socks` feature；环境变量只处理 `EH_PROXY`
  （标准 HTTP(S)_PROXY 交还 reqwest 原生逻辑）。
- 升级 reqwest 0.11 → 0.12（features: json/cookies/socks/gzip）。

### 工程质量

- 统一错误类型 `libeh::error::Error`（thiserror），全库不再返回 `Result<_, String>`；
  库代码零 `panic!`/`unwrap()`/`println!`（测试除外）。
- `EhClientAuth` 手写脱敏 `Debug`（`ipb_pass_hash`/`igneous` 输出 `****`），
  移除库内打印 Cookie 的语句。
- `GIDListItem`/`PageListItem` 改为 `TryFrom`（非法 URL 不再 panic）；
  `Site::url()` 替代 panic 版转换；正则统一 `LazyLock` 编译。
- `config.rs` 的 `mod tests` 补上 `#[cfg(test)]`（此前从未被执行）。
- rustdoc 全面建设：crate/模块/字段级文档、`# Errors`/`# Panics` 段、
  可执行 doctest；`#![warn(missing_docs)]` + CI `RUSTDOCFLAGS="-D warnings"` 门禁。
- 测试：夹具迁移自 EhViewer（10 个列表版式 + 收藏夹 + 详情页，见
  `crates/libeh/tests/fixtures/README.md` 的脱敏记录）；
  网络测试统一 `#[ignore]` + `EH_NETWORK_TESTS=1` 门控；
  CLI 增加参数解析测试。
- CLI（ehrust）：clap 子命令（`search`/`gallery`）、`--dry-run`、
  配置合并（env → yaml → args）、移除配置明文打印。
- CI：`.github/workflows/ci.yml` 与 `.gitea/workflows/ci.yml`
  （fmt → clippy -D warnings → test → doc -D warnings，另有 nightly 网络冒烟）。
- 仓库：`.gitignore` 移除 `Cargo.lock`（workspace 提交 lock 是正确行为）；
  `db.text.json` 移入 `crates/libeh/` 供离线测试；`tags` 结构体去重
  （`url/test.rs` 的重复定义删除）。


### 新增（0.2.0 追加，路线图 P0/P1 首批）

- **图片页解析器** `dto::gallery::page::GalleryPage`：`/s/{pToken}/{gid}-{page}`
  → 图片地址、`showkey`、`skipHathKey`（含 `image_url_with_skip_hath()`
  换源 URL 生成）、原图 `fullimg.php` 链接与 `prompt()` 直链。
- **api.php `showpage`**：`GalleryPageApiRequest` +
  `parse_gallery_page_response`（`i3`/`i6`/`i7` 内嵌 HTML 提取，
  顶层 `error` 识别）+ `EhClient::gallery_page`。
- **分页迭代器** `client::pagination::SearchPager`：`next().await` 逐页
  拉取搜索结果（数字分页），页号自动推进、出错即停。
- **`SearchBuilder::page`**：数字分页参数 `page=N`。
- **种子列表解析器** `dto::torrent::TorrentEntry::parse_page`：
  发布时间/体积/做种数/下载次数；自动剥离下载链接的 `?p=` 私钥参数；
  `EhClient::torrents`。
- **账密登录** `EhClient::login`：论坛 IPB 表单 POST，
  成功后把 `Set-Cookie` 复制写入两个站点域（`dto::signin::parse_sign_in`
  解析昵称与 IPB 错误框）；客户端现持有 `Arc<Jar>` 句柄支持运行期补 Cookie。
- **收藏夹流程**：`EhClient::favorites`（槽位统计 + 画廊列表合并为
  `FavoritesPage`）、`EhClient::add_favorite`（加/删收藏 + 备注）、
  `EhClient::modify_favorites`（批量移动/删除）；表单操作公共底座
  `post_form`（带状态码检查与协议嗅探）。
- **分级超时**（复审 N1）：连接 10s（客户端级）+ 页面 30s / api.php 15s
  （按请求设置），为后续大文件下载预留更长超时的空间。


### 新增（0.2.0 第二批：图片批量下载器 + 复审清尾）

- **图片批量下载器** `client::downloader::Downloader`：
  详情页 → 遍历预览分页收集全部图片页链接 → 并发抓图片页 →
  并发下载落盘 `dest/{gid}/{page:04}.{ext}`；
  `.part` 临时文件 + `Range` 断点续传（206 追加 / 200 重写）；
  并发数 / 覆盖 / 单图超时可配（默认并发 3、300s，站点敏感）；
  `ProgressEvent` 回调实时上报（Planned/Finished/Skipped/Failed）；
  单页失败不中断整体，汇总进 `DownloadSummary::failed`。
- **分级超时补全**：图片下载单图 300s（复审 N1 预留的下载档位）。
- **`EhClient::raw_get`**：暴露原始 GET 构建器供流式下载场景。

### 修复（复审遗留清尾）

- **N2** `EhClient::gallery_tokens`：补检查 api.php 顶层 `error` 字段
  （此前无效 pagelist 会报 serde 错误而非站点原文）。
- **N4** 代理协议白名单收敛到 `EhClientProxy::validate` + 公开常量
  `SUPPORTED_PROTOCOLS`（消除 client.rs 的重复定义）。
- **N6** `SearchBuilder::add_keyword(s)` 过滤空白关键词
  （`Keyword::is_blank()`），空值不再生成无效 `f_search` 片段。
- **N7** `EttHeadMember.when` 反序列化兼容字符串（RFC 3339）与
  数字（毫秒时间戳）两种上游格式。

### 破坏性变更

- 所有 fallible API 的错误类型由 `String` 变为 `libeh::error::Error`；
- `GalleryMetadataRequest::new` 返回 `Result`；
- `GidDataResult::Metadata` 为 `Box<GalleryMetadata>`；
- `PageRange` 字段改为公开的 `start`/`end`；
- `Category::All/Private/Unknown/None` 的 `u16` 转换结果为 `0`；
- 移除 `SearchOptions`、`EhClientAuth::to_string`（改用 `to_pairs`）。
