use scraper::Html;
use serde::{Deserialize, Serialize};

use crate::utils::{
    regex::regex,
    scraper::{parse_to, selector, text_content},
};

const PATTERN_TOTAL_PAGES: &'static str =
    r"Showing ((\d+)(,\d+)*) - ((\d+)(,\d+)*) of (?<total>(\d+)(,\d+)*) images";
const PATTERN_STYLE: &'static str =
    r"background:transparent url\((?<url>[^\(\)]+)\) (?<x>-?\d+)(px)? (?<y>-?\d+)(px)?";

/// GalleryPreview 结构体定义了一个画廊的预览信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryPreview {
    /// 画廊中总共的图片数量
    pub total: i64,
    /// 画廊中所有的预览分页数量
    pub total_set: i64,
    /// 包含了画廊预览的分页信息
    pub pages: Vec<GalleryPreviewPage>,
}

/// GalleryPreviewPage 结构体定义了一个画廊预览页面的信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryPreviewPage {
    // X轴偏移量，单位为像素（px）
    pub offset_x: i64,
    // Y轴偏移量，单位为像素（px）
    pub offset_y: i64,
    // 预览图片的URL
    pub url: String,
    // 预览图片指向的链接
    pub link: String,
}

impl Default for GalleryPreview {
    /// 创建一个新的`GalleryPreview`实例。
    /// 此函数不接受任何参数，直接返回一个初始化的`GalleryPreview`结构体实例。
    fn default() -> Self {
        GalleryPreview {
            total: 0,
            total_set: 0,
            pages: vec![],
        }
    }
}

impl GalleryPreview {
    /// 解析HTML内容，获取图库预览的信息
    pub fn parse(d: &Html) -> Result<Self, String> {
        // 解析总页数
        let total = Self::parse_total_page_count(d)?;
        // 解析总预览集数
        let total_set = Self::parse_total_preview_set(d)?;
        // 解析预览页面列表
        let pages = Self::parse_preview_pages(d)?;

        // 构造GalleryPreview实例
        let gp = GalleryPreview {
            total,
            total_set,
            pages,
        };

        // 返回构造好的GalleryPreview实例
        Ok(gp)
    }

    /// 解析画廊需要预览的总页数
    fn parse_total_page_count(d: &Html) -> Result<i64, String> {
        let r = regex(PATTERN_TOTAL_PAGES)?;
        let s = selector("div.gtb > p.gpc")?;
        let td = match d.select(&s).next() {
            Some(td) => td,
            None => {
                return Err(format!(
                    "Failed to parse total preview pages: {}",
                    "No total preview pages."
                ))
            }
        };
        let text = text_content(td.text());
        match r.captures(&text) {
            Some(caps) => {
                let total = parse_to::<i64>(&caps["total"].replace(",", ""))?;
                Ok(total)
            }
            None => Err(format!(
                "Failed to parse total preview pages: {}",
                "No total preview pages."
            )),
        }
    }

    /// 获取画廊总共需要解析的预览分页数量
    fn parse_total_preview_set(d: &Html) -> Result<i64, String> {
        let s = selector("div.gtb > table.ptt tr > td:nth-last-child(2)")?;
        match d.select(&s).next() {
            Some(td) => {
                let text = text_content(td.text());
                let value = parse_to::<i64>(&text)?;
                Ok(value)
            }
            None => Err(format!(
                "Failed to parse total preview pages: {}",
                "No total preview pages."
            )),
        }
    }

    /// 解析画廊预览页面列表
    pub fn parse_preview_pages(d: &Html) -> Result<Vec<GalleryPreviewPage>, String> {
        // 初始化正则表达式，用于从style属性中提取信息。
        let r = regex(PATTERN_STYLE)?;
        // 定义选择器，用于定位预览页面的div元素。
        let s_div = selector("#gdt > div.gdtm > div")?;
        let s_a = selector("a")?;
        // 创建一个空向量，用于存储解析出的预览页面信息。
        let mut list: Vec<GalleryPreviewPage> = vec![];

        // 遍历所有符合条件的div元素，尝试从中提取预览页面的信息。
        for div in d.select(&s_div) {
            let style = match div.attr("style") {
                Some(style) => style,
                None => continue,
            };
            let caps = match r.captures(&style) {
                Some(caps) => caps,
                None => continue,
            };
            let url = caps["url"].to_string();
            let offset_x = parse_to::<i64>(&caps["x"].to_string())?;
            let offset_y = parse_to::<i64>(&caps["y"].to_string())?;
            let link = match div.select(&s_a).next() {
                Some(a) => a,
                None => continue,
            };
            let link = match link.attr("href") {
                Some(link) => link.to_string(),
                None => continue,
            };
            list.push(GalleryPreviewPage {
                offset_x,
                offset_y,
                url,
                link,
            });
        }
        // 如果解析成功，返回包含所有预览页面信息的向量。
        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    use scraper::Html;

    use super::GalleryPreview;

    #[test]
    fn test_parse_gallery_preview() {
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
        match GalleryPreview::parse(&html) {
            Ok(gp) => {
                println!("{:?}", gp);
            }
            Err(err) => panic!("Failed to parse gallery preview: {}", err),
        }
    }
}
