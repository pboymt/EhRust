# EhRust Structure 项目结构说明

## Introduction 介绍

EhRust is a Rust library for connecting to and interacting with the [E-Hentai](https://e-hentai.org) and [ExHentai](https://exhentai.org) (hereinafter referred to as EH) websites. 

EhRust是一个Rust库，用于向 [E-Hentai](https://e-hentai.org) 与 [ExHentai](https://exhentai.org)（以下简称EH）网站进行连接与交互。

This library encapsulates functions that request API and page content from EH to help you quickly construct URLs and send requests.Also, web parsing helps you quickly process and package key information in EH pages.

这个库封装了请求来自 EH 的 API 和页面内容的函数，能够帮助你快速构建 URL 并发送请求。并且，网页解析功能能够帮助你快速处理并打包 EH 页面中的关键信息。

## Structure 项目结构

### Library / 库 [libeh](crates/libeh/README.md)

- [x] Client / 客户端
  - [x] Authentication / 认证
    - [x] Serialization and Deserialization / 序列化与反序列化
    - [x] Environment Variables / 环境变量读取
  - [x] HTTP Client / HTTP 客户端（`EhClient::try_new`：超时、gzip、状态码检查、协议错误识别）
  - [x] Configuration / 客户端配置
    - [x] Environment Variables / 环境变量读取
    - [x] Serialization and Deserialization / 序列化与反序列化
      - [x] JSON Format / 格式
      - [x] YAML Format / 格式
      - [ ] TOML Format / 格式
  - [x] Proxy / 代理配置（HTTP/HTTPS/SOCKS5）
  - [x] Error / 统一错误类型（thiserror）
- [x] Data Transfer Object / 数据传输对象（搜索结果/画廊详情/评论/预览/收藏夹/api.php）
  - [x] Image Page / 图片页解析（`/s/…` 的图片地址、showkey、换源参数、原图链接；api.php `showpage` 已接线）
- [x] Tag Manager / 标签管理器（EhTagTranslation 数据结构）
- [x] URL Builder / URL 生成器（搜索 + 画廊，含严格/宽松双模式解析）
- [x] Utils / 工具（正则、scraper 辅助、serde 反序列化器）

### Command Line Tool / 命令行工具 [ehrust](crates/ehrust/README.md)

- [x] search 子命令（含 `--dry-run` 离线调试）
- [x] gallery 子命令（详情页摘要）
- [x] 配置合并：环境变量 → YAML → 命令行参数
- [ ] gallerytorrents / archiver 子命令（待实现）

### Extra / 站点能力（0.2.0 新增）

- [x] 种子列表解析（`dto::torrent`，自动剥离 `?p=` 私钥）
- [x] 账密登录（`EhClient::login`，双域 Cookie 补写）
- [x] 收藏夹读取 / 添加 / 批量移动删除
- [x] 搜索分页迭代器（`client::pagination::SearchPager`）
- [x] 按操作分级超时（页面 30s / API 15s）

### Quality Gates / 质量门禁

- [x] `cargo fmt --check`
- [x] `cargo clippy -D warnings`
- [x] `cargo test`（离线夹具测试；网络测试 `#[ignore]` + `EH_NETWORK_TESTS=1`）
- [x] `cargo doc -D warnings`（rustdoc 告警视为错误）
- [x] CI（GitHub Actions / Gitea Actions 双份配置）
