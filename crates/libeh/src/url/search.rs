//! 搜索 URL 构建器：把搜索条件组装为 e-hentai/exhentai 的列表页 URL。
//!
//! 站点搜索的全部参数都挂在列表页的 query string 上，
//! [`SearchBuilder`] 以 builder 模式逐项设置，最终 `build()` 产出 [`Url`]：
//!
//! ```text
//! https://e-hentai.org/watched?f_cats=2&f_search=...&next=1234&advsearch=1&f_sr=on&f_srdd=4
//! └─────────┬─────────┘└┬───┘└───┬────┘└───┬──┘└────┬───┘└────────┬────────┘
//!          站点/订阅    排除分类  关键词    偏移量   高级开关      最低评分
//! ```
//!
//! # 分类过滤（`f_cats`）
//!
//! 站点的 `f_cats` 参数是一个**排除掩码**：置位的分类**不会**出现在结果里，
//! 不发送该参数 = 显示全部分类。[`SearchBuilder`] 内部维护的就是这个排除掩码，
//! 对应的语义化 API 是 [`SearchBuilder::disable_category`] /
//! [`SearchBuilder::enable_category`]。
//!
//! # 高级搜索（`advsearch=1`）
//!
//! 高级开关（已删除、含种子、页数范围、最低评分、禁用过滤器）必须与
//! `advsearch=1` 同时出现才生效，因此调用任一高级项前需要先
//! [`SearchBuilder::enable_advanced_search`]。
//!
//! # 示例
//!
//! ```rust
//! use libeh::dto::gallery::category::Category;
//! use libeh::dto::keyword::Keyword;
//! use libeh::url::search::SearchBuilder;
//!
//! let url = SearchBuilder::new(libeh::dto::site::Site::Eh)
//!     .add_keyword(Keyword::Artist("simon".into()))
//!     .add_keyword(Keyword::Language("chinese".into()))
//!     .disable_category(Category::Misc)
//!     .enable_advanced_search()
//!     .rating(4)
//!     .build()
//!     .unwrap();
//!
//! let query = url.query().unwrap();
//! assert!(query.contains("f_search="));
//! assert!(query.contains("f_cats=1"));           // 排除 Misc（位 0x1）
//! assert!(query.contains("advsearch=1"));
//! assert!(query.contains("f_sr=on&f_srdd=4"));   // 最低评分需要 f_sr=on 才生效
//! ```

use crate::dto::{
    gallery::category::Category, keyword::Keyword, search_offset::Offset, site::Site,
};
use reqwest::Url;

/// 高级搜索中的画廊页数范围。
///
/// `None` 表示该侧不设界；两侧都为 `None` 时不产生任何 query 参数。
/// 需要配合 [`SearchBuilder::enable_advanced_search`] 使用。
///
/// # 示例
///
/// ```rust
/// use libeh::url::search::PageRange;
///
/// let range = PageRange::new(Some(1), Some(10)); // 1–10 页
/// assert_eq!(range.start, Some(1));
/// assert_eq!(range.end, Some(10));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PageRange {
    /// 起始页数（含），`None` 表示不设下界。
    pub start: Option<i64>,
    /// 结束页数（含），`None` 表示不设上界。
    pub end: Option<i64>,
}

impl PageRange {
    /// 构造一个页数范围；`None` 表示对应侧不设界。
    #[must_use]
    pub fn new(start: Option<i64>, end: Option<i64>) -> Self {
        PageRange { start, end }
    }
}

/// 高级搜索选项集合。
///
/// 除 [`AdvancedSearch::enabled`] 外的每个字段对应一个 `f_*` query 参数，
/// 只有在 [`SearchBuilder::enable_advanced_search`] 启用后才会写入 URL。
///
/// 参考站点高级搜索面板：
/// `f_sh`（已删除）、`f_sto`（含种子）、`f_sp`/`f_spf`/`f_spt`（页数范围）、
/// `f_sr`/`f_srdd`（最低评分）、`f_sfl`/`f_sfu`/`f_sft`（禁用对应过滤器）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdvancedSearch {
    /// 是否在 URL 中写入 `advsearch=1` 并启用以下选项。
    pub enabled: bool,
    /// 仅显示已被删除（expunged）的画廊 → `f_sh=on`。
    pub expunged: bool,
    /// 仅显示包含种子的画廊 → `f_sto=on`。
    pub require_torrent: bool,
    /// 画廊页数范围 → `f_sp=on` + `f_spf`/`f_spt`。
    pub between_pages: PageRange,
    /// 最低评分（半星单位，2..=5）→ `f_sr=on` + `f_srdd`。
    pub rating: i8,
    /// 禁用语言过滤器 → `f_sfl=on`。
    pub disable_filters_for_language: bool,
    /// 禁用上传者过滤器 → `f_sfu=on`。
    pub disable_filters_for_uploader: bool,
    /// 禁用标签过滤器 → `f_sft=on`。
    pub disable_filters_for_tags: bool,
}

/// 搜索 URL 构建器。
///
/// 通过链式调用设置搜索条件，最终 [`SearchBuilder::build`] 产出完整 URL。
/// 未设置的项不产生 query 参数（与站点"不发送 = 默认值"的行为一致）。
///
/// 各参数的站点语义见[模块文档](self)。
#[derive(Debug, Clone)]
pub struct SearchBuilder {
    _site: Site,
    _watched: bool,
    _offset: Option<Offset>,
    /// 数字分页页号（0 = 不发送 `page` 参数）。
    _page: usize,
    /// 分类排除掩码（置位 = 排除），即站点 `f_cats` 参数的原值。
    _category: u16,
    _keywords: Vec<Keyword>,
    _advsearch: AdvancedSearch,
}

impl SearchBuilder {
    /// 以指定站点创建一个全默认的构建器。
    ///
    /// 默认不排除任何分类、不带关键词、不带偏移量。
    #[must_use]
    pub fn new(site: Site) -> Self {
        Self {
            _site: site,
            _watched: false,
            _offset: None,
            _page: 0,
            _category: 0,
            _keywords: Vec::new(),
            _advsearch: AdvancedSearch::default(),
        }
    }

    /// 只搜索订阅（watched）的作品，URL 路径改为 `/watched`。
    ///
    /// 该页面需要登录 Cookie，否则站点返回要求登录的页面。
    #[must_use]
    pub fn watched(mut self) -> Self {
        self._watched = true;
        self
    }

    /// 搜索全部作品（取消 [`SearchBuilder::watched`]），URL 路径恢复为 `/`。
    #[must_use]
    pub fn unwatched(mut self) -> Self {
        self._watched = false;
        self
    }

    /// 设置搜索结果的偏移量（`prev`/`next`/`range`/`jump` 参数）。
    #[must_use]
    pub fn offset(mut self, offset: Offset) -> Self {
        self._offset = Some(offset);
        self
    }

    /// 清空偏移量。
    #[must_use]
    pub fn clear_offset(mut self) -> Self {
        self._offset = None;
        self
    }

    /// 设置数字分页页号（query 参数 `page=N`，0 表示不发送 = 第一页）。
    ///
    /// 仅当页面使用 `table.ptt` 数字分页时有效；`searchnav` 快速分页
    /// 页面应改用 [`SearchBuilder::offset`]（`prev`/`next`）。
    /// 与 `offset` 同时设置时两者都会写入 URL，站点以 `page` 为准——
    /// 不要混用。分页迭代见 [`SearchPager`](crate::client::pagination::SearchPager)。
    #[must_use]
    pub fn page(mut self, page: usize) -> Self {
        self._page = page;
        self
    }

    /// 把分类置入排除掩码（该分类不再出现在结果中）。
    ///
    /// # 参数语义
    ///
    /// 仅"真实分类位"（[`Category::Misc`]..[`Category::Western`]）有效；
    /// [`Category::All`] / [`Category::Private`] / [`Category::Unknown`]
    /// 不是有效的 `f_cats` 位，调用它们等效于无操作。
    ///
    /// ```rust
    /// use libeh::dto::gallery::category::Category;
    /// use libeh::url::search::SearchBuilder;
    ///
    /// // 排除 Misc：站点语义为 f_cats=1
    /// let url = SearchBuilder::new(libeh::dto::site::Site::Eh)
    ///     .disable_category(Category::Misc)
    ///     .build().unwrap();
    /// assert_eq!(url.query().unwrap(), "f_cats=1");
    /// ```
    #[must_use]
    pub fn disable_category(mut self, category: Category) -> Self {
        self._category |= u16::from(category);
        self
    }

    /// 把分类移出排除掩码（恢复显示该分类）。
    #[must_use]
    pub fn enable_category(mut self, category: Category) -> Self {
        self._category &= !u16::from(category);
        self
    }

    /// 排除全部分类（`f_cats=1023`，结果为空，通常用于随后只放开少数分类）。
    #[must_use]
    pub fn disable_all_categories(mut self) -> Self {
        self._category = 1023;
        self
    }

    /// 清空排除掩码，显示全部分类。
    #[must_use]
    pub fn enable_all_categories(mut self) -> Self {
        self._category = 0;
        self
    }

    /// 切换分类的排除状态：已排除则恢复，未排除则排除。
    #[must_use]
    pub fn toggle_category(mut self, category: Category) -> Self {
        if (self._category & u16::from(category)) == u16::from(category) {
            self._category &= !u16::from(category);
        } else {
            self._category |= u16::from(category);
        }
        self
    }

    /// 启用高级搜索（在 URL 写入 `advsearch=1`）。
    ///
    /// 页数范围、最低评分等高级项必须在此之后才会写入 URL。
    #[must_use]
    pub fn enable_advanced_search(mut self) -> Self {
        self._advsearch.enabled = true;
        self
    }

    /// 高级搜索：仅显示已被删除（expunged）的画廊 → `f_sh=on`。
    #[must_use]
    pub fn browse_expunged_galleries(mut self) -> Self {
        self._advsearch.expunged = true;
        self
    }

    /// 高级搜索：仅显示包含种子的画廊 → `f_sto=on`。
    #[must_use]
    pub fn require_gallery_torrent(mut self) -> Self {
        self._advsearch.require_torrent = true;
        self
    }

    /// 高级搜索：设置画廊页数范围 → `f_sp=on` + `f_spf`/`f_spt`。
    #[must_use]
    pub fn between_pages(mut self, page_range: PageRange) -> Self {
        self._advsearch.between_pages = page_range;
        self
    }

    /// 高级搜索：设置最低评分。
    ///
    /// 参数取站点 `f_srdd` 的原始值（半星单位）：`2`=1 星、`3`=1.5 星、
    /// `4`=2 星、`5`=2.5 星；超出 `2..=5` 的取值被忽略（保持原值）。
    /// 写入 URL 时同时携带 `f_sr=on`，缺少它站点会忽略评分筛选。
    #[must_use]
    pub fn rating(mut self, rating: i8) -> Self {
        if (2..=5).contains(&rating) {
            self._advsearch.rating = rating;
        }
        self
    }

    /// 高级搜索：禁用语言过滤器 → `f_sfl=on`。
    #[must_use]
    pub fn disable_filters_for_language(mut self) -> Self {
        self._advsearch.disable_filters_for_language = true;
        self
    }

    /// 高级搜索：禁用上传者过滤器 → `f_sfu=on`。
    #[must_use]
    pub fn disable_filters_for_uploader(mut self) -> Self {
        self._advsearch.disable_filters_for_uploader = true;
        self
    }

    /// 高级搜索：禁用标签过滤器 → `f_sft=on`。
    #[must_use]
    pub fn disable_filters_for_tags(mut self) -> Self {
        self._advsearch.disable_filters_for_tags = true;
        self
    }

    /// 添加一个关键词（按 `f_search` 中的空格分隔参与检索）。
    #[must_use]
    pub fn add_keyword(mut self, keyword: Keyword) -> Self {
        self._keywords.push(keyword);
        self
    }

    /// 批量添加关键词。
    #[must_use]
    pub fn add_keywords(mut self, keywords: Vec<Keyword>) -> Self {
        self._keywords.extend(keywords);
        self
    }

    /// 获取当前分类排除掩码（即写入 `f_cats` 的原值；0 = 不排除）。
    #[must_use]
    pub fn category(&self) -> u16 {
        self._category
    }

    /// 基础 URL：站点根，或订阅模式下的 `/watched` 路径。
    fn build_base_url(&self) -> Result<Url, crate::error::Error> {
        let mut url = Site::url(self._site)?;
        if self._watched {
            url.set_path("/watched");
        }
        Ok(url)
    }

    /// 追加数字分页参数 `page`（0 = 不发送，代表第一页）。
    fn build_append_page(&self, mut url: Url) -> Url {
        if self._page > 0 {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("page", &self._page.to_string());
        }
        url
    }

    /// 追加分类排除掩码 `f_cats`（0 = 不发送，代表全部显示）。
    fn build_append_category(&self, mut url: Url) -> Url {
        if self._category != 0 {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("f_cats", &self._category.to_string());
        }
        url
    }

    /// 追加偏移量参数（`prev`/`next`/`range`，及可选的 `jump`）。
    fn build_append_offset(&self, mut url: Url) -> Url {
        if let Some(offset) = self._offset.clone() {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.extend_pairs(offset);
        }
        url
    }

    /// 追加关键词参数 `f_search`（多个关键词以空格连接后整体 URL 编码）。
    fn build_append_keywords(&self, mut url: Url) -> Url {
        if !self._keywords.is_empty() {
            let keyword_list: Vec<String> = self
                ._keywords
                .iter()
                .map(std::string::ToString::to_string)
                .collect();
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("f_search", keyword_list.join(" ").as_str());
        }
        url
    }

    /// 追加高级搜索参数；未启用高级搜索时此函数为空操作。
    fn build_append_advanced_search(&self, mut url: Url) -> Url {
        if !self._advsearch.enabled {
            return url;
        }
        {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("advsearch", "1");
            if self._advsearch.expunged {
                query_pairs.append_pair("f_sh", "on");
            }
            if self._advsearch.require_torrent {
                query_pairs.append_pair("f_sto", "on");
            }
            // 页数范围必须带 f_sp=on，否则站点忽略 f_spf/f_spt
            let range = &self._advsearch.between_pages;
            if range.start.is_some() || range.end.is_some() {
                query_pairs.append_pair("f_sp", "on");
                if let Some(spf) = range.start {
                    query_pairs.append_pair("f_spf", &spf.to_string());
                }
                if let Some(spt) = range.end {
                    query_pairs.append_pair("f_spt", &spt.to_string());
                }
            }
            // 最低评分必须带 f_sr=on，否则站点忽略 f_srdd
            if self._advsearch.rating > 0 {
                query_pairs.append_pair("f_sr", "on");
                query_pairs.append_pair("f_srdd", &self._advsearch.rating.to_string());
            }
            if self._advsearch.disable_filters_for_language {
                query_pairs.append_pair("f_sfl", "on");
            }
            if self._advsearch.disable_filters_for_uploader {
                query_pairs.append_pair("f_sfu", "on");
            }
            if self._advsearch.disable_filters_for_tags {
                query_pairs.append_pair("f_sft", "on");
            }
        }
        url
    }

    /// 构建最终 URL。
    ///
    /// # Errors
    ///
    /// 仅当站点无法转为 URL（[`Site::Un`]）时返回 [`Error::Config`](crate::error::Error::Config)；
    /// 其余参数不参与 URL 解析，不会失败。
    pub fn build(self) -> Result<Url, crate::error::Error> {
        let url = self.build_base_url()?;
        let url = self.build_append_offset(url);
        let url = self.build_append_page(url);
        let url = self.build_append_category(url);
        let url = self.build_append_keywords(url);
        Ok(self.build_append_advanced_search(url))
    }
}

impl Default for SearchBuilder {
    /// 等价于 [`SearchBuilder::new(Site::Eh)`](SearchBuilder::new)。
    fn default() -> Self {
        SearchBuilder::new(Site::Eh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::site::Site;

    #[test]
    fn default_build_is_site_root() {
        let url = SearchBuilder::new(Site::Eh).build().unwrap();
        assert_eq!(url.as_str(), "https://e-hentai.org/");
        assert!(url.query().is_none());
    }

    #[test]
    fn rating_requires_f_sr_on() {
        let url = SearchBuilder::new(Site::Eh)
            .enable_advanced_search()
            .rating(4)
            .build()
            .unwrap();
        let q = url.query().unwrap();
        assert!(q.contains("f_sr=on"));
        assert!(q.contains("f_srdd=4"));
    }

    #[test]
    fn rating_out_of_range_ignored() {
        let url = SearchBuilder::new(Site::Eh)
            .enable_advanced_search()
            .rating(1)
            .build()
            .unwrap();
        // 1 不是合法的 f_srdd 值，应保持未设置
        assert!(!url.query().unwrap_or_default().contains("f_srdd"));
    }

    #[test]
    fn page_range_requires_f_sp_on() {
        let url = SearchBuilder::new(Site::Eh)
            .enable_advanced_search()
            .between_pages(PageRange::new(Some(1), Some(10)))
            .build()
            .unwrap();
        let q = url.query().unwrap();
        assert!(q.contains("f_sp=on"));
        assert!(q.contains("f_spf=1"));
        assert!(q.contains("f_spt=10"));
    }

    #[test]
    fn category_mask_semantics() {
        // 排除 Doujinshi → f_cats=2
        let url = SearchBuilder::new(Site::Eh)
            .disable_category(Category::Doujinshi)
            .build()
            .unwrap();
        assert_eq!(url.query().unwrap(), "f_cats=2");

        // 先排除全部再放开 Doujinshi → f_cats=1021（只显示 Doujinshi）
        let url = SearchBuilder::new(Site::Eh)
            .disable_all_categories()
            .enable_category(Category::Doujinshi)
            .build()
            .unwrap();
        assert_eq!(url.query().unwrap(), "f_cats=1021");

        // 无效掩码位不写入 URL
        let url = SearchBuilder::new(Site::Eh)
            .disable_category(Category::Unknown)
            .build()
            .unwrap();
        assert!(url.query().is_none());
    }

    #[test]
    fn watched_path_and_offset() {
        let url = SearchBuilder::new(Site::Eh)
            .watched()
            .offset(Offset::Next(1234, None))
            .build()
            .unwrap();
        assert_eq!(url.path(), "/watched");
        assert!(url.query().unwrap().contains("next=1234"));
    }
}
