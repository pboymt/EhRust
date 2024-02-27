use std::str::FromStr;

use chrono::{DateTime, NaiveDateTime, Utc};
use scraper::{element_ref::Text, Selector};

use super::regex::regex;

pub fn selector(selector: &str) -> Result<Selector, String> {
    match Selector::parse(selector) {
        Ok(selector) => Ok(selector),
        Err(err) => Err(format!("Failed to parse selector: {}", err)),
    }
}

pub fn text_content(text: Text) -> String {
    let seq: Vec<String> = text.map(|t| t.trim().to_string()).collect();
    let seq: Vec<String> = seq.into_iter().filter(|s| !s.is_empty()).collect();
    seq.join(" ").to_string()
}

pub fn parse_posted(text: &str) -> Result<DateTime<Utc>, String> {
    match NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M") {
        Ok(date) => Ok(date.and_utc()),
        Err(err) => Err(format!("Failed to parse posted: {}", err)),
    }
}

struct FavoriteSlotRgba(i16, i16, i16);

fn parse_style_color(text: &str) -> Result<FavoriteSlotRgba, String> {
    let r = regex(r"background-color:rgba\((\d+),(\d+),(\d+),")?;
    match r.captures(text) {
        Some(caps) => {
            let r = caps[1].parse::<i16>();
            let g = caps[2].parse::<i16>();
            let b = caps[3].parse::<i16>();
            match (r, g, b) {
                (Ok(r), Ok(g), Ok(b)) => Ok(FavoriteSlotRgba(r, g, b)),
                _ => Err(format!("Failed to parse style color: {}", "No color.")),
            }
        }
        None => Err(format!("Failed to parse style color: {}", "No color.")),
    }
}

pub fn parse_favorite_slot(text: &str) -> Result<isize, String> {
    let rgb = parse_style_color(text)?;
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
            return Err(format!(
                "Failed to parse favorite slot: {}",
                "Unknown slot."
            ))
        }
    };
    Ok(slot)
}

pub fn parse_rating(text: &str) -> Result<f32, String> {
    let r = regex(r"-?(\d+)px -?(\d+)px")?;
    match r.captures(text) {
        Some(caps) => {
            let rating: f32 = 5.0;
            let major = caps[1].parse::<i32>().unwrap();
            let patch = caps[2].parse::<i32>().unwrap();
            let rating = rating - major as f32 / 16.0;
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

pub fn parse_to<T: FromStr>(value: &str) -> Result<T, String> {
    let result = value.parse::<T>();
    match result {
        Ok(value) => Ok(value),
        Err(_) => Err(format!("Failed to parse to number: {}", value)),
    }
}
