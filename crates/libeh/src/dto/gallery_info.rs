use serde::{Deserialize, Serialize};

use crate::dto::{category::Category, keyword::Keyword};
use chrono::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryInfo {
    /// 画廊 ID
    pub gid: i64,
    /// 画廊版本令牌
    pub token: String,
    /// 画廊标题
    pub title: String,
    /// 画廊标题（日文）
    pub title_jpn: String,
    /// 画廊缩略图
    pub thumb: String,
    /// 画廊类型
    pub category: Category,
    /// 上传时间
    pub posted: DateTime<Utc>,
    /// 上传者
    pub uploader: Option<String>,
    /// 画廊评分
    pub rating: f32,
    /// 画廊标签列表
    pub tags: Vec<Keyword>,
    /// 画廊页数
    pub pages: i64,
    /// 收藏夹
    pub favorite_slot: isize,
}

impl Default for GalleryInfo {
    fn default() -> Self {
        Self {
            gid: -1,
            token: "".to_string(),
            title: "".to_string(),
            title_jpn: "".to_string(),
            thumb: "".to_string(),
            category: Category::Unknown,
            posted: DateTime::<Utc>::MIN_UTC,
            uploader: None,
            rating: -1.0,
            tags: vec![],
            pages: -1,
            favorite_slot: -1,
        }
    }
}
