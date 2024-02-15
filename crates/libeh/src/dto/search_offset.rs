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

impl IntoIterator for Offset {
    type Item = (String, String);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut result = match self {
            Offset::Prev(gid, _) => vec![("prev".to_string(), gid.to_string())],
            Offset::Next(gid, _) => vec![("next".to_string(), gid.to_string())],
            Offset::Range(percent) => vec![("range".to_string(), percent.to_string())],
        };
        match self {
            Offset::Prev(_, Some(jump)) => result.push(("jump".to_string(), jump.to_string())),
            Offset::Next(_, Some(jump)) => result.push(("jump".to_string(), jump.to_string())),
            _ => {}
        };
        result.into_iter()
    }
}
