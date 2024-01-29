/** 站点类型 */
#[derive(Debug, Clone, Copy)]
pub enum Site {
    Eh,
    Ex,
}

/** 搜索结果的偏移量 */
#[derive(Debug, Clone)]
pub enum Offset {
    /** 在指定 gid 后发布的画廊。 */
    Prev(i64, Option<String>),
    /** 在指定 gid 前发布的画廊。 */
    Next(i64, Option<String>),
    /** 指定范围内某个百分比（0-98，新到旧）的作品。 */
    Range(i64),
}

/** 搜索结果的类型 */
#[derive(Debug, Clone, Copy)]
pub enum Category {
    Misc = 1,
    Doujinshi = 2,
    Manga = 4,
    ArtistCG = 8,
    GameCG = 16,
    ImageSet = 32,
    Cosplay = 64,
    // AsianPorn = 128,
    NonH = 256,
    Western = 512,
}

#[derive(Debug, Clone)]
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
}

impl From<Category> for u16 {
    fn from(category: Category) -> u16 {
        match category {
            Category::Misc => 1,
            Category::Doujinshi => 2,
            Category::Manga => 4,
            Category::ArtistCG => 8,
            Category::GameCG => 16,
            Category::ImageSet => 32,
            Category::Cosplay => 64,
            Category::NonH => 256,
            Category::Western => 512,
        }
    }
}
