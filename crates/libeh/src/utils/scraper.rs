use std::str::FromStr;

use chrono::{DateTime, NaiveDateTime, Utc};
use scraper::{element_ref::Text, Selector};

use super::regex::regex;

/// 根据给定的选择器字符串创建一个选择器对象
pub fn selector(selector: &str) -> Result<Selector, String> {
    match Selector::parse(selector) {
        Ok(selector) => Ok(selector),
        Err(err) => Err(format!("Failed to parse selector: {}", err)),
    }
}

/// 将文本内容转换为字符串
pub fn text_content(text: Text) -> String {
    // 将文本内容转换为字符串的向量
    let seq: Vec<String> = text.map(|t| t.trim().to_string()).collect();
    // 过滤掉空字符串
    let seq: Vec<String> = seq.into_iter().filter(|s| !s.is_empty()).collect();
    // 将向量中的字符串连接起来，并去除首尾空格
    seq.join(" ").trim().to_string()
}

/// 解析画廊的发布时间
pub fn parse_posted(text: &str) -> Result<DateTime<Utc>, String> {
    // 尝试从字符串中解析出一个日期时间
    match NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M") {
        // 如果解析成功，则将日期时间转换为UTC时间并返回
        Ok(date) => Ok(date.and_utc()),
        // 如果解析失败，则返回解析错误信息
        Err(err) => Err(format!("Failed to parse posted: {}", err)),
    }
}

/// 收藏夹槽位颜色
struct FavoriteSlotRgba(i16, i16, i16);

/// 解析样式颜色
fn parse_style_color(text: &str) -> Result<FavoriteSlotRgba, String> {
    // 匹配rgba颜色值的正则表达式
    let r = regex(r"background-color:rgba\((\d+),(\d+),(\d+),")?;
    // 匹配到颜色值
    match r.captures(text) {
        // 匹配成功
        Some(caps) => {
            // 解析红色值
            let r = caps[1].parse::<i16>();
            // 解析绿色值
            let g = caps[2].parse::<i16>();
            // 解析蓝色值
            let b = caps[3].parse::<i16>();
            // 解析成功
            match (r, g, b) {
                (Ok(r), Ok(g), Ok(b)) => Ok(FavoriteSlotRgba(r, g, b)),
                // 解析失败
                _ => Err(format!("Failed to parse style color: {}", "No color.")),
            }
        }
        // 匹配失败
        None => Err(format!("Failed to parse style color: {}", "No color.")),
    }
}

/// 解析收藏夹槽位
pub fn parse_favorite_slot(text: &str) -> Result<isize, String> {
    // 解析颜色
    let rgb = parse_style_color(text)?;

    // 根据颜色值匹配槽位
    let slot = match rgb {
        FavoriteSlotRgba(0, 0, 0) => 0,
        FavoriteSlotRgba(240, 0, 0) => 1,
        FavoriteSlotRgba(240, 160, 0) => 2,
        FavoriteSlotRgba(208, 208, 0) => 3,
        FavoriteSlotRgba(0, 128, 0) => 4,
        FavoriteSlotRgba(144, 240, 64) => 5,
        FavoriteSlotRgba(64, 176, 240) => 6,
        FavoriteSlotRgba(0, 0, 240) => 7,
        FavoriteSlotRgba(80, 0, 128) => 8,
        FavoriteSlotRgba(224, 128, 224) => 9,
        _ => {
            // 未知槽位
            return Err(format!(
                "Failed to parse favorite slot: {}",
                "Unknown slot."
            ));
        }
    };
    // 返回槽位
    Ok(slot)
}

pub fn parse_rating(text: &str) -> Result<f32, String> {
    // 使用正则表达式匹配文本中的数字
    let r = regex(r"-?(\d+)px -?(\d+)px")?;
    // 匹配成功后，获取匹配到的数字
    match r.captures(text) {
        Some(caps) => {
            // 初始化评分值为5.0
            let rating: f32 = 5.0;
            // 解析第一个数字为i32类型
            let major = caps[1].parse::<i32>().unwrap();
            // 解析第二个数字为i32类型
            let patch = caps[2].parse::<i32>().unwrap();
            // 计算主要评分值
            let rating = rating - major as f32 / 16.0;
            // 如果补丁号为21，则减去0.5
            if patch == 21 {
                let rating = rating - 0.5;
                Ok(rating)
            } else {
                Ok(rating)
            }
        }
        None => Err(format!("Failed to parse rating: {}", "No rating.")),
    }
}

/// 转换字符串为指定类型，类型需要实现 FromStr trait
pub fn parse_to<T: FromStr>(value: &str) -> Result<T, String> {
    // 尝试将字符串解析为指定类型
    let result = value.parse::<T>();
    // 根据解析结果进行处理
    match result {
        // 解析成功，返回解析结果
        Ok(value) => Ok(value),
        // 解析失败，返回错误信息
        Err(_) => Err(format!("Failed to parse to number: {}", value)),
    }
}
