//! 搜索结果的分页迭代器。
//!
//! [`SearchPager`] 把"构建 URL → 请求 → 解析 → 取下一页页号"的循环
//! 封装为 `next().await` 的拉取式迭代器，逐页返回 [`SearchResult`]：
//!
//! - 第一页使用构建器原样（`page` 参数为 0，即不发送）；
//! - 之后的页号取自上一页结果解析出的 `next_page`（`table.ptt` 数字
//!   分页）；`next_page` 缺失或不再推进时迭代结束；
//! - 任何网络/解析错误都会终止迭代并把错误作为最后一项返回，
//!   避免调用方死循环。
//!
//! 只支持**数字分页**。`searchnav` 快速分页（收藏夹等）没有数字页号，
//! 请自行解析 `next_href` 并重新发起请求。
//!
//! # 示例
//!
//! ```no_run
//! use libeh::client::{client::EhClient, config::EhClientConfig, pagination::SearchPager};
//! use libeh::dto::keyword::Keyword;
//! use libeh::url::search::SearchBuilder;
//!
//! # async fn demo() -> Result<(), libeh::error::Error> {
//! let client = EhClient::try_new(EhClientConfig::env()?)?;
//! let builder = SearchBuilder::new(libeh::dto::site::Site::Eh)
//!     .add_keyword(Keyword::Artist("simon".into()));
//!
//! let mut pager = SearchPager::new(client, builder);
//! while let Some(result) = pager.next().await {
//!     let result = result?;
//!     println!("page {} : {} galleries", result.next_page, result.gallery_info_list.len());
//!     if result.next_page > 3 { break; } // 最多看 4 页
//! }
//! # Ok(())
//! # }
//! ```

use crate::client::client::EhClient;
use crate::dto::search_result::SearchResult;
use crate::error::Error;
use crate::url::search::SearchBuilder;

/// 搜索结果的分页迭代器。
///
/// 构造与使用见[模块文档](self)；`SearchPager` 消耗 [`EhClient`]
/// （[`EhClient`] 是廉价的 `Arc` 句柄，克隆即可继续使用）。
#[derive(Debug)]
pub struct SearchPager {
    client: EhClient,
    builder: SearchBuilder,
    /// 下一页的页号（0 基）。
    next_page: usize,
    /// 迭代是否已结束（翻完、无法推进或出错）。
    finished: bool,
}

impl SearchPager {
    /// 以客户端与搜索条件创建分页迭代器（从第一页开始）。
    ///
    /// 构建器中已设置的 `page` 会被重置为 0。
    #[must_use]
    pub fn new(client: EhClient, builder: SearchBuilder) -> Self {
        Self {
            client,
            builder: builder.page(0),
            next_page: 0,
            finished: false,
        }
    }

    /// 取下一页结果；迭代结束时返回 `None`。
    ///
    /// 出错时（网络、解析）返回 `Some(Err(..))` 并终止后续迭代。
    pub async fn next(&mut self) -> Option<Result<SearchResult, Error>> {
        if self.finished {
            return None;
        }
        let url = match self.builder.clone().page(self.next_page).build() {
            Ok(url) => url,
            Err(e) => {
                self.finished = true;
                return Some(Err(e));
            }
        };
        let html = match self.client.get_html(url).await {
            Ok(html) => html,
            Err(e) => {
                self.finished = true;
                return Some(Err(e));
            }
        };
        match SearchResult::parse(html) {
            Ok(result) => {
                // next_page 未推进（缺失、或站点已到末页）则终止
                match usize::try_from(result.next_page) {
                    Ok(np) if result.next_page > self.next_page as isize => {
                        self.next_page = np;
                    }
                    _ => self.finished = true,
                }
                Some(Ok(result))
            }
            Err(e) => {
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::client::client::EhClient;
    use crate::client::config::EhClientConfig;
    use crate::client::pagination::SearchPager;
    use crate::url::search::SearchBuilder;

    #[test]
    fn pager_resets_start_page() {
        // 构建器自带的 page 会被重置：迭代必须从第一页开始
        let client = EhClient::new(EhClientConfig::default());
        let builder = SearchBuilder::new(crate::dto::site::Site::Eh).page(3);
        let pager = SearchPager::new(client, builder);
        assert_eq!(pager.next_page, 0);
        assert!(!pager.finished);
    }
}
