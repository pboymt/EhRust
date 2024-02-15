use reqwest::Url;
use serde::{Deserialize, Serialize};

/** 站点类型 */
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Site {
    #[serde(rename = "un")]
    Un,
    #[serde(rename = "eh")]
    Eh,
    #[serde(rename = "ex")]
    Ex,
}

impl From<String> for Site {
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
    fn from(value: Site) -> Self {
        match value {
            Site::Eh => Url::parse("https://e-hentai.org/").unwrap(),
            Site::Ex => Url::parse("https://exhentai.org/").unwrap(),
            _ => panic!("Unrecognized site."),
        }
    }
}
