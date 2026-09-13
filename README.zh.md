# EhRust - 使用 Rust 编写的，简单、易用、快速的 E-Hentai 与 ExHentai 库

EhRust 是一个简单的、易于使用的、快速的 Rust 库，用于 **E-Hentai** 与 **ExHentai**。

# 功能

- 用于生成搜索与画廊 URL 的 URL 构建器，便于使用。
- 便于使用的 API，用于搜索与画廊。
- 内建的关键字构建器与解析器。
- 内建的搜索结果解析器（兼容五种列表版式与两种分页结构）。
- 内建的画廊解析器（详情 / 评论 / 预览）。
- 认证 Cookie 注入（双域 + `nw=1`）、代理（HTTP/SOCKS5）、统一错误类型。

# 快速上手

```rust,no_run
use libeh::client::{client::EhClient, config::EhClientConfig};
use libeh::dto::keyword::Keyword;

# async fn demo() -> Result<(), libeh::error::Error> {
// 读取环境变量配置（EH_SITE / EH_PROXY / EH_AUTH_ID / EH_AUTH_HASH / EH_AUTH_IGNEOUS）
let config = EhClientConfig::env()?;
let client = EhClient::try_new(config)?;

// 结构化搜索：等价于站点搜索 artist:"simon$" language:"chinese$"
let result = client
    .search_parsed(
        vec![Keyword::Artist("simon".into()), Keyword::Language("chinese".into())],
        None,
    )
    .await?;

for info in &result.gallery_info_list {
    println!("{} ({} 页)", info.title, info.pages);
}
# Ok(())
# }
```

更多示例见 `crates/libeh` 的 [rustdoc 文档](https://github.com/pboymt/EhRust)（`cargo doc --open`）。

# 命令行工具

`ehrust` 提供 `search` / `gallery` 子命令：

```console
$ ehrust search -k "artist:simon" -k "language:chinese" --site eh
$ ehrust gallery https://e-hentai.org/g/2791585/3e7e1c7107/
$ ehrust search -k "female:big breasts" --dry-run   # 仅打印搜索 URL
```

配置优先级：命令行参数 > `--config` 指定的 YAML（缺省 `config.yaml`）> 环境变量。

# 测试

```console
$ cargo test                 # 全部离线测试（夹具迁移自 EhViewer）
$ EH_NETWORK_TESTS=1 cargo test -- --ignored   # 真实网络冒烟测试（需可用连接/代理）
```

# 许可证

GPL-3.0。测试夹具迁移自 EhViewer（同为 GPL-3.0），详见 `crates/libeh/tests/fixtures/README.md`。
