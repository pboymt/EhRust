# EhRust - Simple, Easy and Speedy Rust Library for E-Hentai and ExHentai

EhRust is a simple, easy-to-use, and speedy Rust library for the **E-Hentai** and **ExHentai**.

[中文文档](./README.zh.md)

## Features

- Convenient URL builder for searching and gallery.
- Easy-to-use API for searching and gallery.
- Built-in keyword builder and parser (namespace-aware, colon-safe).
- Built-in search results parser (five list layouts, two pagination styles).
- Built-in gallery parser (detail / comments / previews).
- Cookie authentication (dual-domain + `nw=1`), HTTP/SOCKS5 proxy, typed errors.

## Quick start

```rust,no_run
use libeh::client::{client::EhClient, config::EhClientConfig};
use libeh::dto::keyword::Keyword;

# async fn demo() -> Result<(), libeh::error::Error> {
// Reads EH_SITE / EH_PROXY / EH_AUTH_ID / EH_AUTH_HASH / EH_AUTH_IGNEOUS
let config = EhClientConfig::env()?;
let client = EhClient::try_new(config)?;

// Structured search: equivalent to artist:"simon$" language:"chinese$"
let result = client
    .search_parsed(
        vec![Keyword::Artist("simon".into()), Keyword::Language("chinese".into())],
        None,
    )
    .await?;

for info in &result.gallery_info_list {
    println!("{} ({} pages)", info.title, info.pages);
}
# Ok(())
# }
```

See the [rustdoc](https://github.com/pboymt/EhRust) (`cargo doc --open`) for the full guide.

## CLI

The `ehrust` binary ships `search` / `gallery` subcommands:

```console
$ ehrust search -k "artist:simon" -k "language:chinese" --site eh
$ ehrust gallery https://e-hentai.org/g/2791585/3e7e1c7107/
$ ehrust search -k "female:big breasts" --dry-run   # print the search URL only
```

Configuration precedence: CLI args > `--config` YAML (default `config.yaml`) > environment variables.

## Testing

```console
$ cargo test                 # offline tests (fixtures ported from EhViewer)
$ EH_NETWORK_TESTS=1 cargo test -- --ignored   # real-network smoke tests
```

## License

GPL-3.0. Test fixtures are ported from EhViewer (also GPL-3.0); see
`crates/libeh/tests/fixtures/README.md`.
