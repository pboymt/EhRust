# 测试夹具（fixtures）

## 来源

本目录下的 HTML/JSON 夹具迁移自 [EhViewer（Ehviewer_CN_SXJ）](https://github.com/xiaojieonly/Ehviewer_CN_SXJ)
的单元测试资源 `app/src/test/resources/com/hippo/ehviewer/client/parser/`，
用于驱动 `libeh` 各解析器的离线回归测试。

两个项目均以 GPL-3.0 授权，夹具的复制与再分发符合许可证要求。

## 覆盖范围

| 文件 | 用途 | 使用方 |
|---|---|---|
| `GalleryListParserTestE*.html` / `GalleryListParserTestEx*.html`（10 个） | E 站/里站 × Minimal/MinimalPlus/Compact/Extended/Thumbnail 五种列表版式（`table.ptt` 数字分页） | `tests/search_result_ptt.rs` |
| `FavoritesListParser.html` | 收藏夹页（`searchnav` 快速分页 + 10 个分类槽位） | `tests/search_result_searchnav.rs` |
| `EmptyGalleryList.html` | 空结果视图（过滤过严/页码越界） | `tests/search_result_searchnav.rs` |
| `GalleryDetail.html` | 画廊详情页（含内联脚本参数、键值表、标签、评论、预览） | `tests/gallery_detail.rs` |
| `pending/` | 其余页面夹具（Torrent 列表、TopList、home、以图搜图等），供后续功能开发使用 | 暂无 |

## 脱敏记录

- `GalleryDetail.html`：`var apiuid = 4596468`（抓取者会话参数）→ 改写为 `1000000`；
- 全目录已 grep 验证不含 `ipb_member_id` / `ipb_pass_hash` / `igneous` / `Set-Cookie` 等凭据。

## 命名说明

EhViewer 原 Java 测试引用的夹具文件名（如
`GalleryListParserTestEMinimal.GalleryTopListEX.html`）与磁盘实际文件名不一致
（测试已年久失修）；本目录使用磁盘实际文件名，测试断言沿用 Java 测试的意图。
