//! 画廊基本信息。
//!
//! [`GalleryInfo`](crate::dto::gallery::info::GalleryInfo) 是贯穿列表页（[`crate::dto::search_result`]）、
//! 详情页（[`crate::dto::gallery::detail`]）与 api.php
//! （[`crate::dto::api::GalleryMetadata`]）的最小画廊描述单元；
//! 字段在 HTML 路径可能缺失（保持 [`GalleryInfo::default`](crate::dto::gallery::info::GalleryInfo::default) 值），
//! 在 API 路径总是完整。
//!
//! # 时区说明
//!
//! [`GalleryInfo::posted`](crate::dto::gallery::info::GalleryInfo::posted) 在 HTML 路径按**页面渲染文本以 UTC 解释**
//! （站点按账号时区渲染，可能有偏差）；API 路径来自 unix 时间戳，
//! 是精确的 UTC。需要原始渲染文本时取 [`GalleryInfo::posted_raw`](crate::dto::gallery::info::GalleryInfo::posted_raw)。

use serde::{Deserialize, Serialize};

use crate::dto::{gallery::category::Category, keyword::Keyword};
use chrono::prelude::*;

/// 画廊基本信息。
///
/// 各字段在列表页/详情页/API 三条路径中的可用性见[模块文档](self)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryInfo {
    /// 画廊 ID。
    pub gid: i64,
    /// 画廊版本令牌（10 位十六进制）。
    pub token: String,
    /// 画廊标题（罗马字/英文）。
    pub title: String,
    /// 画廊标题（日文）；列表页不提供（空串），详情页/API 提供。
    pub title_jpn: String,
    /// 画廊缩略图 URL。
    pub thumb: String,
    /// 画廊类型（解析自分类文本；解析失败为 [`Category::None`]）。
    pub category: Category,
    /// 上传时间（时区语义见[模块文档](self)#时区说明）。
    pub posted: DateTime<Utc>,
    /// 页面渲染的上传时间原文（仅 HTML 路径提供，API 路径为 `None`）。
    pub posted_raw: Option<String>,
    /// 上传者；被 disown、缩略图版式或 API 未提供时为 `None`。
    pub uploader: Option<String>,
    /// 画廊评分（0.0–5.0；未评分为 `-1.0`）。
    pub rating: f32,
    /// 画廊标签列表。
    pub tags: Vec<Keyword>,
    /// 画廊页数。
    pub pages: i64,
    /// 收藏夹：`0..=9` 为云端收藏夹编号，`-1` 为本地收藏，
    /// `-2` 为未知/未收藏（与 EhViewer 语义一致）。
    pub favorite_slot: isize,
}

impl Default for GalleryInfo {
    /// 全默认实例：gid=-1、空标题、[`Category::None`]、评分 -1.0、
    /// 页数 -1、收藏槽位 -2（未知/未收藏）。
    fn default() -> Self {
        Self {
            gid: -1,
            token: String::new(),
            title: String::new(),
            title_jpn: String::new(),
            thumb: String::new(),
            category: Category::None,
            posted: DateTime::<Utc>::MIN_UTC,
            posted_raw: None,
            uploader: None,
            rating: -1.0,
            tags: vec![],
            pages: -1,
            favorite_slot: -2,
        }
    }
}
