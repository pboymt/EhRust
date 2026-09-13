//! 数据传输对象（DTO）与对应的 HTML/JSON 解析器。
//!
//! 按数据来源划分：
//!
//! - 来自 **api.php**（JSON）：[`api`](crate::dto::api)；
//! - 来自 **页面 HTML**：[`search_result`](crate::dto::search_result)（列表/搜索页）、
//!   [`favorites`](crate::dto::favorites)（收藏夹槽位）、[`gallery`](crate::dto::gallery)（详情/评论/预览）；
//! - **纯结构**：[`keyword`](crate::dto::keyword)（搜索关键词）、[`site`](crate::dto::site)（站点）、
//!   [`search_offset`](crate::dto::search_offset)（结果偏移量）。
//!
//! 解析器均为纯函数（HTML 字符串 → 结构体），可离线测试；
//! 解析失败统一返回 [`Error::Parse`](crate::error::Error::Parse) 并携带原始片段。

/// 对 api.e-hentai.org 的 api 请求与响应的数据封装
pub mod api;
/// 收藏夹页面的分类槽位信息
pub mod favorites;
/// 画廊
pub mod gallery;
/// 搜索关键词，各类标签的枚举及其转换方法
pub mod keyword;
/// 搜索结果的偏移量
pub mod search_offset;
/// 搜索结果及解析器
pub mod search_result;
/// 站点类型枚举
pub mod site;
