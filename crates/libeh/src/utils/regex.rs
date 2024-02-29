use regex::Regex;

/// 生成正则表达式对象
pub fn regex(regex: &str) -> Result<Regex, String> {
    match Regex::new(regex) {
        Ok(regex) => Ok(regex),
        Err(err) => Err(format!("Failed to parse regex: {}", err)),
    }
}
