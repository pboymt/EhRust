//! 画廊评论的解析器。
//!
//! 输入为详情页 HTML（评论位于 `#cdiv` 内的 `.c1` 块），
//! 每条评论提取：
//!
//! - **身份**：评论 ID（`comment_score_{id}`）、发布者（`.c3` 内的锚点）、
//!   发布时间（`Posted on … by:`，按 UTC 解释，见时区说明）；
//! - **操作位**（`.c4`）：`Vote+`/`Vote-`/`Edit` 按钮是否存在、
//!   是否已投过票（按钮带行内样式即为已投）；
//! - **分数与投票明细**：`.c5` 的分数、`.c7` 的 `Base N` 底分与
//!   逐用户投票列表（`user ±N`），以及 `and N more...` 的折叠数；
//! - **正文**：`.c6` 的原始 HTML。
//!
//! # 时区说明
//!
//! 评论时间按站点渲染文本解析（`%d %B %Y, %H:%M`，英文月份）
//! 并**按 UTC 解释**；站点按账号时区渲染，可能与真实 UTC 时间有偏差。
//!
//! # 容错
//!
//! 单条评论内的小结构缺失（如无投票、无分数）不产生错误，
//! 仅整体结构异常（无 `.c3`）时该条目失败。
//!
//! # 示例
//!
//! ```rust,no_run
//! # use libeh::dto::gallery::comment::GalleryComment;
//! # use scraper::Html;
//! # fn demo(html: &str) -> Result<(), libeh::error::Error> {
//! let d = Html::parse_document(html);
//! for comment in GalleryComment::parse(&d)? {
//!     println!("{} ({}): {:?}", comment.user, comment.score, comment.id);
//! }
//! # Ok(())
//! # }
//! ```

use chrono::{DateTime, NaiveDateTime, Utc};
use scraper::Html;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::utils::{
    regex::regex,
    scraper::{parse_to, selector, text_content},
};

/// `Posted on … by:` 中的时间部分。
const PATTERN_COMMENT_TIME: &str = r"Posted on (.+) by:";
/// `comment_score_{id}` 中的评论 ID。
const PATTERN_COMMENT_ID: &str = r"comment_score_(\d+)";
/// `.c7` 中的底分（`Base +N` / `Base -N`）。
const PATTERN_COMMENT_VOTE_BASE: &str = r"Base ([\+\-]?\d+)";
/// `.c7` 中逐用户投票（`user +N` / `user -N`）。
const PATTERN_COMMENT_VOTE: &str = r"(?<user>.+) (?<score>[\+\-]?\d+)$";
/// `.c7` 中的折叠投票数（`and N more...`）。
const PATTERN_COMMENT_VOTE_MORE: &str = r"and (\d+) more...";

/// 画廊评论。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryComment {
    /// 评论 ID（`comment_score_{id}`；未渲染分数区时为 `None`）。
    pub id: Option<i64>,
    /// 评论分数（底分 + 明细票之和）。
    pub score: i64,
    /// 是否可编辑（`.c4` 中存在 `Edit` 按钮）。
    pub editable: bool,
    /// 是否可投赞成票（`.c4` 中存在 `Vote+` 按钮）。
    pub can_vote_up: bool,
    /// 是否已投过赞成票（`Vote+` 按钮带行内样式）。
    pub voted_up: bool,
    /// 是否可投反对票（`.c4` 中存在 `Vote-` 按钮）。
    pub can_vote_down: bool,
    /// 是否已投过反对票（`Vote-` 按钮带行内样式）。
    pub voted_down: bool,
    /// 评论投票状态（底分与逐用户明细）。
    pub vote_state: GalleryCommentVoteState,
    /// 评论时间（页面渲染文本按 UTC 解释）。
    pub time: DateTime<Utc>,
    /// 评论用户。
    pub user: String,
    /// 评论内容（原始 HTML）。
    pub comment: String,
    /// 最后编辑时间（`.c8` 存在时尝试解析，解析失败为 `None`）。
    pub last_edited: Option<DateTime<Utc>>,
}

/// 评论投票状态（`.c7` 区域）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GalleryCommentVoteState {
    /// 底分（`Base N`，通常是上传者与标签贡献）。
    pub base: i64,
    /// 可见的逐用户投票明细。
    pub votes: Vec<GalleryCommentVote>,
    /// 因折叠未显示的投票数（`and N more...`）。
    pub more: i64,
}

/// 单条投票明细。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryCommentVote {
    /// 投票用户。
    pub user: String,
    /// 投票分值（+1 / -1）。
    pub score: i64,
}

impl GalleryComment {
    /// 创建一条全默认的评论。
    #[must_use]
    pub fn new() -> GalleryComment {
        GalleryComment {
            id: None,
            score: 0,
            editable: false,
            can_vote_up: false,
            voted_up: false,
            can_vote_down: false,
            voted_down: false,
            vote_state: GalleryCommentVoteState::default(),
            time: Utc::now(),
            user: String::new(),
            last_edited: None,
            comment: String::new(),
        }
    }

    /// 解析详情页中的全部评论。
    ///
    /// # Errors
    ///
    /// 页面存在 `#cdiv` 但某条评论缺少 `.c3`（时间/用户区）或投票
    /// 明细无法解析时返回 [`Error::Parse`]；页面没有评论区时返回空列表。
    pub fn parse(d: &Html) -> Result<Vec<GalleryComment>, Error> {
        let r_comment_time = regex(PATTERN_COMMENT_TIME)?;
        let r_comment_id = regex(PATTERN_COMMENT_ID)?;
        let r_comment_vote_base = regex(PATTERN_COMMENT_VOTE_BASE)?;
        let r_comment_vote = regex(PATTERN_COMMENT_VOTE)?;
        let r_comment_vote_more = regex(PATTERN_COMMENT_VOTE_MORE)?;
        let s = selector("#cdiv")?;
        let mut comments: Vec<GalleryComment> = vec![];
        if let Some(cdiv) = d.select(&s).next() {
            let s = selector("div.c1")?;
            for c1 in cdiv.select(&s) {
                let mut gc = GalleryComment::new();
                let s = selector("div.c3")?;
                let Some(c3) = c1.select(&s).next() else {
                    return Err(Error::parse_msg(
                        "gallery comment",
                        "invalid comment (no c3)",
                        String::new(),
                    ));
                };
                let text = text_content(c3.text());
                // 评论时间
                gc.time = match r_comment_time.captures(&text) {
                    Some(caps) => Self::parse_comment_time(&caps[1])?,
                    None => {
                        return Err(Error::parse_msg(
                            "gallery comment",
                            "no comment datetime",
                            text,
                        ))
                    }
                };
                // 评论用户（.c3 内第一个锚点）
                let s = selector("a")?;
                match c1.select(&s).next() {
                    Some(anchor) => gc.user = text_content(anchor.text()),
                    None => {
                        return Err(Error::parse_msg("gallery comment", "no comment user", text))
                    }
                }

                // 操作位（.c4）：Vote+ / Vote- / Edit
                if let Ok(s) = selector("div.c4") {
                    if let Some(c4) = c1.select(&s).next() {
                        for e in c4.children().filter(|c| c.value().is_element()) {
                            let el = scraper::ElementRef::wrap(e);
                            let Some(el) = el else { continue };
                            match text_content(el.text()).as_str() {
                                "Vote+" => {
                                    gc.can_vote_up = true;
                                    gc.voted_up = !el
                                        .value()
                                        .attr("style")
                                        .unwrap_or_default()
                                        .trim()
                                        .is_empty();
                                }
                                "Vote-" => {
                                    gc.can_vote_down = true;
                                    gc.voted_down = !el
                                        .value()
                                        .attr("style")
                                        .unwrap_or_default()
                                        .trim()
                                        .is_empty();
                                }
                                "Edit" => gc.editable = true,
                                _ => {}
                            }
                        }
                    }
                }

                // 评论 ID 与分数
                let s = selector(r#"span[id^="comment_score_"]"#)?;
                if let Some(comment_score) = c1.select(&s).next() {
                    if let Some(id_text) = comment_score.attr("id") {
                        if let Some(caps) = r_comment_id.captures(id_text) {
                            gc.id = Some(parse_to::<i64>(&caps[1])?);
                        }
                    }
                    let text = text_content(comment_score.text());
                    gc.score = parse_to::<i64>(&text)?;
                }

                // 评论正文（原始 HTML）
                let s = selector(r#"div.c6[id^="comment_"]"#)?;
                if let Some(c6) = c1.select(&s).next() {
                    gc.comment = c6.inner_html().trim().to_string();
                }

                // 投票状态（.c7）：Base N + 逐用户明细 + and N more...
                let s = selector(r#"div.c7[id^="cvotes_"]"#)?;
                if let Some(c7) = c1.select(&s).next() {
                    let mut c7text = c7.text();
                    if let Some(base) = c7text.next() {
                        match r_comment_vote_base.captures(base) {
                            Some(caps) => {
                                gc.vote_state.base = parse_to::<i64>(&caps[1])?;
                            }
                            None => {
                                return Err(Error::parse_msg(
                                    "gallery comment vote",
                                    "invalid vote base",
                                    base,
                                ))
                            }
                        }
                        let s = selector("span")?;
                        for vote in c7.select(&s) {
                            let text = text_content(vote.text());
                            match r_comment_vote.captures(text.trim()) {
                                Some(caps) => {
                                    let user = caps[1].to_string();
                                    let score = parse_to::<i64>(&caps[2])?;
                                    gc.vote_state.votes.push(GalleryCommentVote { user, score });
                                }
                                None => {
                                    return Err(Error::parse_msg(
                                        "gallery comment vote",
                                        "invalid vote entry",
                                        text,
                                    ))
                                }
                            }
                        }
                        if let Some(more) = c7text.last() {
                            if let Some(caps) = r_comment_vote_more.captures(more) {
                                gc.vote_state.more = parse_to::<i64>(&caps[1]).unwrap_or(0);
                            }
                        }
                    }
                }
                comments.push(gc);
            }
        }
        Ok(comments)
    }

    /// 解析评论时间（`%d %B %Y, %H:%M`，英文月份，按 UTC 解释）。
    ///
    /// # Errors
    ///
    /// 格式不匹配时返回 [`Error::Parse`]。
    pub fn parse_comment_time(text: &str) -> Result<DateTime<Utc>, Error> {
        NaiveDateTime::parse_from_str(text.trim(), "%d %B %Y, %H:%M")
            .map(|naive| naive.and_utc())
            .map_err(|err| Error::parse("comment time", err.to_string(), text))
    }
}

impl Default for GalleryComment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    use scraper::Html;

    use super::GalleryComment;

    fn load_fixture(name: &str) -> String {
        let path = format!(
            "{}/tests/fixtures/parser/{name}",
            env!("CARGO_MANIFEST_DIR")
        );
        let mut file = File::open(&path).unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        buf
    }

    #[test]
    fn test_parse_gallery_comments() {
        let html = load_fixture("GalleryDetail.html");
        let html = Html::parse_document(&html);
        let comments = GalleryComment::parse(&html).unwrap();
        assert!(!comments.is_empty());
        for comment in &comments {
            // 有明细且无折叠时，底分 + 明细之和应等于总分
            if comment.vote_state.more == 0 {
                let all_votes = comment.vote_state.base
                    + comment
                        .vote_state
                        .votes
                        .iter()
                        .map(|v| v.score)
                        .sum::<i64>();
                assert_eq!(
                    all_votes, comment.score,
                    "vote mismatch for comment {comment:?}"
                );
            }
        }
    }
}
