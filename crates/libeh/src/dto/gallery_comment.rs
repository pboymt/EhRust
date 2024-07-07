use chrono::{DateTime, NaiveDateTime, Utc};
use scraper::Html;
use serde::{Deserialize, Serialize};

use crate::utils::{
    regex::regex,
    scraper::{parse_to, selector, text_content},
};

const PATTERN_COMMENT_TIME: &'static str = r"Posted on (.+) by:";
const PATTERN_COMMENT_ID: &'static str = r"comment_score_(\d+)";
const PATTERN_COMMENT_VOTE_BASE: &'static str = r"Base ([\+\-]?\d+)";
const PATTERN_COMMENT_VOTE: &'static str = r"(?<user>.+) (?<score>[\+\-]?\d+)$";
const PATTERN_COMMENT_VOTE_MORE: &'static str = r"and (\d+) more...";

/// 画廊评论
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryComment {
    /// 评论 ID
    pub id: Option<i64>,
    /// 评论分数
    pub score: i64,
    /// 是否可编辑
    pub editable: bool,
    /// 是否可投票
    pub can_vote_up: bool,
    /// 是否已投票
    pub voted_up: bool,
    /// 是否可投票
    pub can_vote_down: bool,
    /// 是否已投票
    pub voted_down: bool,
    /// 评论投票状态
    pub vote_state: GalleryCommentVoteState,
    /// 评论时间
    pub time: DateTime<Utc>,
    /// 评论用户
    pub user: String,
    /// 评论内容
    pub comment: String,
    /// 最后编辑时间
    pub last_edited: Option<DateTime<Utc>>,
}

/// 画廊评论投票状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryCommentVoteState {
    pub base: i64,
    pub votes: Vec<GalleryCommentVote>,
    pub more: i64,
}

/// 画廊评论投票
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryCommentVote {
    pub user: String,
    pub score: i64,
}

impl GalleryCommentVoteState {
    pub fn new() -> Self {
        GalleryCommentVoteState {
            base: 0,
            votes: vec![],
            more: 0,
        }
    }
}

impl GalleryComment {
    pub fn new() -> GalleryComment {
        GalleryComment {
            id: None,
            score: 0,
            editable: false,
            can_vote_up: false,
            voted_up: false,
            can_vote_down: false,
            voted_down: false,
            vote_state: GalleryCommentVoteState::new(),
            time: Utc::now(),
            user: String::new(),
            last_edited: None,
            comment: String::new(),
        }
    }

    /// 解析画廊评论
    pub fn parse(d: &Html) -> Result<Vec<GalleryComment>, String> {
        let r_comment_time = regex(PATTERN_COMMENT_TIME)?;
        let r_comment_id = regex(PATTERN_COMMENT_ID)?;
        let r_comment_vote_base = regex(PATTERN_COMMENT_VOTE_BASE)?;
        let r_comment_vote = regex(PATTERN_COMMENT_VOTE)?;
        let r_comment_vote_more = regex(PATTERN_COMMENT_VOTE_MORE)?;
        let s = selector("#cdiv")?;
        let mut comments: Vec<GalleryComment> = vec![];
        if let Some(cdiv) = d.select(&s).next() {
            let s = selector("div.c1")?;
            // 遍历评论
            for c1 in cdiv.select(&s) {
                let mut gc = GalleryComment::new();
                let s = selector("div.c3")?;
                match c1.select(&s).next() {
                    Some(c3) => {
                        let text = text_content(c3.text());
                        // 解析评论发布时间
                        gc.time = match r_comment_time.captures(&text) {
                            Some(caps) => Self::parse_comment_time(&caps[1])?,
                            None => {
                                return Err(format!(
                                    "Failed to parse comment: {}",
                                    "No comment datetime."
                                ))
                            }
                        };
                        // 解析评论发布用户
                        let s = selector("a")?;
                        match c1.select(&s).next() {
                            Some(c3) => {
                                let text = text_content(c3.text());
                                gc.user = text;
                            }
                            None => {
                                return Err(format!(
                                    "Failed to parse comment: {}",
                                    "No comment user."
                                ))
                            }
                        };
                    }
                    None => return Err(format!("Failed to parse comment: {}", "Invalid comment.")),
                }
                // 解析评论分数
                let s = selector(r#"span[id^="comment_score_"]"#)?;
                match c1.select(&s).next() {
                    Some(comment_score) => {
                        let text = comment_score.attr("id");
                        if let Some(text) = text {
                            match r_comment_id.captures(&text) {
                                Some(caps) => {
                                    gc.id = Some(parse_to::<i64>(&caps[1])?);
                                }
                                None => {
                                    return Err(format!(
                                        "Failed to parse comment: {}",
                                        "Invalid comment id."
                                    ));
                                }
                            }
                        }
                        let text = text_content(comment_score.text());
                        gc.score = parse_to::<i64>(&text)?;
                    }
                    None => {}
                }
                // 解析评论内容
                let s = selector(r#"div.c6[id^="comment_""#)?;
                match c1.select(&s).next() {
                    Some(c6) => {
                        let text = c6.inner_html().trim().to_string();
                        gc.comment = text;
                    }
                    None => {}
                }
                // 解析评论评分情况
                let s = selector(r#"div.c7[id^="cvotes_""#)?;
                match c1.select(&s).next() {
                    Some(c7) => {
                        let mut c7text = c7.text();
                        if let Some(base) = c7text.next() {
                            match r_comment_vote_base.captures(&base) {
                                Some(caps) => {
                                    let score = parse_to::<i64>(&caps[1])?;
                                    gc.vote_state.base = score;
                                }
                                None => {
                                    return Err(format!(
                                        "Failed to parse comment vote: {}",
                                        "Invalid comment vote base."
                                    ));
                                }
                            }
                            let s = selector("span")?;
                            for vote in c7.select(&s) {
                                let text = text_content(vote.text());
                                match r_comment_vote.captures(text.trim()) {
                                    Some(caps) => {
                                        let user = caps[1].to_string();
                                        let score = parse_to::<i64>(&caps[2])?;
                                        let vote = GalleryCommentVote { user, score };
                                        gc.vote_state.votes.push(vote);
                                    }
                                    None => {
                                        return Err(format!(
                                            "Failed to parse comment vote: {}",
                                            "Invalid comment vote."
                                        ));
                                    }
                                }
                            }
                            if let Some(more) = c7text.last() {
                                match r_comment_vote_more.captures(&more) {
                                    Some(caps) => {
                                        let more = parse_to::<i64>(&caps[1])?;
                                        gc.vote_state.more = more;
                                    }
                                    None => {}
                                }
                            }
                        }
                    }
                    None => {}
                }
                comments.push(gc);
            }
        }
        Ok(comments)
    }

    /// 解析评论时间
    fn parse_comment_time(text: &str) -> Result<DateTime<Utc>, String> {
        match NaiveDateTime::parse_from_str(text, "%d %B %Y, %H:%M") {
            Ok(date) => Ok(date.and_utc()),
            Err(err) => Err(format!("Failed to parse comment time: {}", err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    use scraper::Html;

    use super::GalleryComment;

    #[test]
    fn test_parse_gallery_comments() {
        let mut cwd = std::env::current_dir().unwrap();
        cwd.push("../../samples/gallery_old.html");
        let mut file = match File::open(cwd) {
            Ok(file) => file,
            Err(err) => panic!("Failed to open file: {}", err),
        };
        let html = {
            let mut buf = String::new();
            match file.read_to_string(&mut buf) {
                Ok(_) => buf,
                Err(err) => panic!("Failed to read file: {}", err),
            }
        };
        let html = Html::parse_document(&html);
        let comments = GalleryComment::parse(&html).unwrap();
        println!("Comments: {}", comments.len());
        for comment in comments {
            println!("Comment: {:?}", comment);
            if comment.vote_state.more == 0 {
                let all_votes = comment.vote_state.base
                    + comment
                        .vote_state
                        .votes
                        .iter()
                        .map(|v| v.score)
                        .sum::<i64>();
                assert_eq!(all_votes, comment.score);
            }
        }
    }
}
