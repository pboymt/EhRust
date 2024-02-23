use reqwest::Url;
use serde::{Deserialize, Serialize};

/// 站点类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Site {
    /// 未知站点
    #[serde(rename = "un")]
    Un,
    /// e-hentai.org
    #[serde(rename = "eh")]
    Eh,
    /// exhentai.org
    #[serde(rename = "ex")]
    Ex,
}

impl From<String> for Site {
    /// 将域名转换为站点类型
    fn from(value: String) -> Self {
        if value.contains("e-hentai.org") {
            Site::Eh
        } else if value.contains("exhentai.org") {
            Site::Ex
        } else {
            Site::Un
        }
    }
}

impl From<Site> for Url {
    /// 将站点类型转换为 Url
    fn from(value: Site) -> Self {
        match value {
            Site::Eh => Url::parse("https://e-hentai.org/").unwrap(),
            Site::Ex => Url::parse("https://exhentai.org/").unwrap(),
            _ => panic!("Unrecognized site."),
        }
    }
}
