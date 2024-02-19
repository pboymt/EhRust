use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// 搜索关键词
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Keyword {
    /** 一般的搜索关键词。 */
    Normal(String),
    /** 作品的语言。 */
    Language(String),
    /** 同人作品模仿的原始作品。 */
    Parody(String),
    /** 作品中出现的角色。 */
    Character(String),
    /** 绘画作者/写手。 */
    Artist(String),
    /** 角色扮演者。 */
    Cosplayer(String),
    /** 制作社团或公司。 */
    Group(String),
    /** 女性角色相关的恋物标签。 */
    Female(String),
    /** 男性角色相关的恋物标签。 */
    Male(String),
    /** 两性/中性的恋物标签。 */
    Mixed(String),
    /** 经过确认的技术标签。 */
    Other(String),
    /** 用于分类出错的图库，当某个重新分类标签权重达到 100，将移动图库至对应分类。 */
    Reclass(String),
    /** 尚未正式添加至 E-Hentai 标签系统的标签。在提供翻译前，需要在 E-Hentai 论坛发帖将该标签移动到合适的命名空间。 */
    Temp(String),
    /** 上传者名称 */
    Uploader(String),
}

impl ToString for Keyword {
    fn to_string(&self) -> String {
        match self {
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
            Keyword::Uploader(keyword) => format!("uploader:\"{}$\"", keyword),
        }
    }
}

impl From<Keyword> for String {
    fn from(keyword: Keyword) -> String {
        keyword.to_string()
    }
}

impl From<String> for Keyword {
    fn from(s: String) -> Keyword {
        let result = Keyword::from_str(&s);
        result.unwrap_or(Keyword::Normal(s))
    }
}

impl FromStr for Keyword {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result: Vec<&str> = s.split(":").collect();
        match result.len() {
            1 => Ok(Keyword::Normal(result[0].into())),
            2 => match result[0] {
                "language" => Ok(Keyword::Language(result[1].into())),
                "parody" => Ok(Keyword::Parody(result[1].into())),
                "character" => Ok(Keyword::Character(result[1].into())),
                "artist" => Ok(Keyword::Artist(result[1].into())),
                "cosplayer" => Ok(Keyword::Cosplayer(result[1].into())),
                "group" => Ok(Keyword::Group(result[1].into())),
                "female" => Ok(Keyword::Female(result[1].into())),
                "male" => Ok(Keyword::Male(result[1].into())),
                "mixed" => Ok(Keyword::Mixed(result[1].into())),
                "other" => Ok(Keyword::Other(result[1].into())),
                "reclass" => Ok(Keyword::Reclass(result[1].into())),
                "temp" => Ok(Keyword::Temp(result[1].into())),
                "uploader" => Ok(Keyword::Uploader(result[1].into())),
                _ => Err(format!("Invalid keyword: {}", s)),
            },
            _ => Err(format!("Invalid keyword: {}", s)),
        }
    }
}
