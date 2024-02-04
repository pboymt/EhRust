use std::vec;

use reqwest::Url;

use super::enums::{Category, Keyword, Offset, Site};

#[derive(Debug, Clone)]
pub struct PageRange(Option<i64>, Option<i64>);

#[derive(Debug, Clone)]
pub struct AdvancedSearch {
    pub enabled: bool,
    pub expunged: bool,
    pub require_torrent: bool,
    pub between_pages: PageRange,
    pub rating: i8,
    pub disable_filters_for_language: bool,
    pub disable_filters_for_uploader: bool,
    pub disable_filters_for_tags: bool,
}

#[derive(Debug, Clone)]
pub struct SearchBuilder {
    _site: Site,
    _offset: Option<Offset>,
    _category: u16,
    _keywords: Vec<Keyword>,
    _advsearch: AdvancedSearch,
}

impl SearchBuilder {
    pub fn new(site: Site) -> Self {
        // 创建一个新的实例并设置_site为site参数
        // 设置_offset为None
        // 设置_category为0
        // 设置_keywords为一个空的向量
        // 创建一个AdvancedSearch实例并设置enabled为false
        // 设置expunged为false
        // 设置require_torrent为false
        // 设置between_pages为PageRange(None, None)
        // 设置rating为0
        // 设置disable_filters_for_language为false
        // 设置disable_filters_for_uploader为false
        // 设置disable_filters_for_tags为false
        Self {
            _site: site,
            _offset: None,
            _category: 0,
            _keywords: vec![],

            _advsearch: AdvancedSearch {
                enabled: false,
                expunged: false,
                require_torrent: false,
                between_pages: PageRange(None, None),
                rating: 0,
                disable_filters_for_language: false,
                disable_filters_for_uploader: false,
                disable_filters_for_tags: false,
            },
        }
    }

    /// 获取当前对象的主机名
    fn host(&self) -> String {
        // 根据_site的值选择对应的主机名
        match self._site {
            // 如果_site为Site::Eh，则主机名为"e-hentai.org"
            Site::Eh => "e-hentai.org".to_string(),
            // 如果_site为Site::Ex，则主机名为"exhentai.org"
            Site::Ex => "exhentai.org".to_string(),
        }
    }

    /// 设置搜索的偏移量
    pub fn offset(mut self, offset: Offset) -> SearchBuilder {
        self._offset = Some(offset);
        self
    }

    /// 启用/禁用类别
    pub fn toggle_category(mut self, category: Category) -> SearchBuilder {
        // 判断当前类别是否已禁用
        let disabled = (self._category & u16::from(category) as u16) == u16::from(category) as u16;
        // 如果已禁用，则去除该类别的标志位
        if disabled {
            self._category = self._category & (1023 ^ u16::from(category) as u16);
        } else {
            // 如果未禁用，则设置该类别的标志位
            self._category |= u16::from(category) as u16;
        }
        self
    }

    /// 禁用所有类别
    pub fn mask_all_categories(mut self) -> SearchBuilder {
        self._category = 1023;
        self
    }

    /// 添加关键词
    pub fn keyword(mut self, keyword: Keyword) -> SearchBuilder {
        self._keywords.push(keyword);
        self
    }

    /// 启用高级搜索
    pub fn enable_advanced_search(mut self) -> SearchBuilder {
        self._advsearch.enabled = true;
        self
    }

    /// 仅浏览已删除的画廊
    pub fn browse_expunged_galleries(mut self) -> SearchBuilder {
        self._advsearch.expunged = true;
        self
    }

    /// 只搜索包含种子的画廊
    pub fn require_gallery_torrent(mut self) -> SearchBuilder {
        self._advsearch.require_torrent = true;
        self
    }

    /// 设置画廊包含的页数范围
    pub fn between_pages(mut self, page_range: PageRange) -> SearchBuilder {
        self._advsearch.between_pages = page_range;
        self
    }

    /// 设置画廊的最低评分
    pub fn rating(mut self, rating: i8) -> SearchBuilder {
        if rating >= 0 && rating <= 5 {
            self._advsearch.rating = rating;
        }
        self
    }

    /// 禁用语言过滤器
    pub fn disable_filters_for_language(mut self) -> SearchBuilder {
        self._advsearch.disable_filters_for_language = true;
        self
    }

    /// 禁用上传者过滤器
    pub fn disable_filters_for_uploader(mut self) -> SearchBuilder {
        self._advsearch.disable_filters_for_uploader = true;
        self
    }

    /// 禁用标签过滤器
    pub fn disable_filters_for_tags(mut self) -> SearchBuilder {
        self._advsearch.disable_filters_for_tags = true;
        self
    }

    /// 获取当前类别
    pub fn category(&self) -> u16 {
        self._category
    }

    /// 获取基础URL
    fn base_url(&self) -> Result<Url, String> {
        let host = self.host();
        let url_result = Url::parse(format!("https://{}", host).as_str());
        match url_result {
            Ok(url) => Ok(url),
            Err(err) => Err(format!("解析URL失败: {}", err)),
        }
    }

    /// 向URL中追加分类信息
    fn append_category(&self, mut url: Url) -> Result<Url, String> {
        if self._category != 0 {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("f_cats", &self._category.to_string());
        }
        Ok(url)
    }

    /// 在给定的URL后面追加偏移量
    fn append_offset(&self, mut url: Url) -> Result<Url, String> {
        // 如果_offset存在
        if let Some(offset) = self._offset.clone() {
            // 根据_offset的类型进行匹配
            match offset {
                // 如果_offset是Next类型
                Offset::Next(gid, jump) => {
                    // 获取url的query_pairs_mut方法的返回值
                    let mut query_pairs = url.query_pairs_mut();
                    // 向query_pairs中添加键值对，键为"next"，值为gid的字符串形式
                    query_pairs.append_pair("next", &gid.to_string());
                    // 如果jump存在
                    if let Some(jump) = jump {
                        // 向query_pairs中添加键值对，键为"jump"，值为jump
                        query_pairs.append_pair("jump", &jump);
                    }
                }
                // 如果_offset是Prev类型
                Offset::Prev(gid, jump) => {
                    // 获取url的query_pairs_mut方法的返回值
                    let mut query_pairs = url.query_pairs_mut();
                    // 向query_pairs中添加键值对，键为"prev"，值为gid的字符串形式
                    query_pairs.append_pair("prev", &gid.to_string());
                    // 如果jump存在
                    if let Some(jump) = jump {
                        // 向query_pairs中添加键值对，键为"jump"，值为jump
                        query_pairs.append_pair("jump", &jump);
                    }
                }
                // 如果_offset是Range类型
                Offset::Range(range) => {
                    // 获取url的query_pairs_mut方法的返回值
                    let mut query_pairs = url.query_pairs_mut();
                    // 向query_pairs中添加键值对，键为"range"，值为range的字符串形式
                    query_pairs.append_pair("range", &range.to_string());
                }
            }
        }
        Ok(url)
    }

    /// 向URL中追加关键词
    fn append_keywors(&self, mut url: Url) -> Result<Url, String> {
        // 创建一个空的关键词列表
        let mut keyword_list: Vec<String> = vec![];
        // 遍历关键词列表
        for keyword in &self._keywords {
            // 根据关键词类型生成对应的关键词字符串
            let keyword_unit = match keyword {
                Keyword::Normal(keyword) => format!("\"{}\"", keyword),
                Keyword::Language(keyword) => format!("l:\"{}$\"", keyword),
                Keyword::Parody(keyword) => format!("p:\"{}$\"", keyword),
                Keyword::Character(keyword) => format!("c:\"{}$\"", keyword),
                Keyword::Artist(keyword) => format!("a:\"{}$\"", keyword),
                Keyword::Cosplayer(keyword) => format!("cos:\"{}$\"", keyword),
                Keyword::Group(keyword) => format!("g:\"{}$\"", keyword),
                Keyword::Female(keyword) => format!("f:\"{}$\"", keyword),
                Keyword::Male(keyword) => format!("m:\"{}$\"", keyword),
                Keyword::Mixed(keyword) => format!("x:\"{}$\"", keyword),
                Keyword::Other(keyword) => format!("o:\"{}$\"", keyword),
                Keyword::Reclass(keyword) => format!("r:\"{}$\"", keyword),
                Keyword::Temp(keyword) => format!("temp:\"{}$\"", keyword),
            };
            // 将关键词字符串添加到关键词列表中
            keyword_list.push(keyword_unit);
        }
        // 将关键词列表追加到URL的查询参数中
        {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("f_search", keyword_list.join(" ").as_str());
        }
        // 返回更新后的URL
        Ok(url)
    }

    fn append_advanced_search(&self, mut url: Url) -> Result<Url, String> {
        // 如果高级搜索功能已启用
        if self._advsearch.enabled {
            // 获取可修改的查询参数
            let mut query_pairs = url.query_pairs_mut();
            // 添加"advsearch=1"查询参数
            query_pairs.append_pair("advsearch", "1");
            // 如果已删除搜索结果，则添加"f_sh=on"查询参数
            if self._advsearch.expunged {
                query_pairs.append_pair("f_sh", "on");
            }
            // 如果要求只显示种子文件，则添加"f_sto=on"查询参数
            if self._advsearch.require_torrent {
                query_pairs.append_pair("f_sto", "on");
            }
            // 根据分页范围添加相应的查询参数
            match self._advsearch.between_pages {
                // 分页范围为指定起始页码和结束页码
                PageRange(Some(spf), Some(spt)) => {
                    query_pairs.append_pair("f_spf", &spf.to_string());
                    query_pairs.append_pair("f_spt", &spt.to_string());
                }
                // 分页范围为指定起始页码，无结束页码
                PageRange(Some(spf), None) => {
                    query_pairs.append_pair("f_spf", &spf.to_string());
                }
                // 分页范围为无起始页码，指定结束页码
                PageRange(None, Some(spt)) => {
                    query_pairs.append_pair("f_spt", &spt.to_string());
                }
                // 分页范围为无起始页码和结束页码
                PageRange(None, None) => {}
            }
            // 如果搜索结果的评分要求大于0，则添加"f_srdd"查询参数
            if self._advsearch.rating > 0 {
                query_pairs.append_pair("f_srdd", &self._advsearch.rating.to_string());
            }
            // 如果已禁用语言过滤器，则添加"f_sfl=on"查询参数
            if self._advsearch.disable_filters_for_language {
                query_pairs.append_pair("f_sfl", "on");
            }
            // 如果已禁用上传者过滤器，则添加"f_sfu=on"查询参数
            if self._advsearch.disable_filters_for_uploader {
                query_pairs.append_pair("f_sfu", "on");
            }
            // 如果已禁用标签过滤器，则添加"f_sft=on"查询参数
            if self._advsearch.disable_filters_for_tags {
                query_pairs.append_pair("f_sft", "on");
            }
        }
        Ok(url)
    }

    pub fn build(self) -> Result<Url, String> {
        let mut url = self.base_url()?;
        url = self.append_offset(url)?;
        url = self.append_category(url)?;
        url = self.append_keywors(url)?;
        url = self.append_advanced_search(url)?;
        Ok(url)
    }
}

impl Default for SearchBuilder {
    fn default() -> Self {
        SearchBuilder::new(Site::Eh)
    }
}
