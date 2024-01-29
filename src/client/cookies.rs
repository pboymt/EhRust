use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;
use std::sync::Mutex;

use cookie::Cookie;
use reqwest::cookie::CookieStore;
use reqwest::Url;

pub type NameMap = HashMap<String, String>;
pub type PathMap = HashMap<String, NameMap>;
pub type DomainMap = HashMap<String, PathMap>;

pub struct Store {
    map: Mutex<DomainMap>,
}

fn set_cookies(
    map: &mut DomainMap,
    cookie_headers: &mut dyn Iterator<Item = &reqwest::header::HeaderValue>,
    url: &Url,
) {
    let url_domain = url.domain().map_or(None, |s| Some(s.to_string()));
    let url_path = url.path().to_string();
    for header in cookie_headers {
        let header_str = match header.to_str() {
            Ok(header) => header,
            Err(_) => continue,
        };
        let parsed_cookie = match Cookie::from_str(header_str) {
            Ok(cookie) => cookie,
            Err(_) => continue,
        };
        println!("set cookie: {:#?}", parsed_cookie);
        let domain = if let Some(domain) = parsed_cookie.domain() {
            domain.to_string()
        } else {
            url_domain.clone().unwrap_or_default()
        };
        if !map.contains_key(&domain) {
            map.insert(domain.clone(), PathMap::new());
        }
        let path_map = map.get_mut(&domain).unwrap();
        let path = if let Some(path) = parsed_cookie.path() {
            path.to_string()
        } else {
            url_path.clone()
        };
        if !path_map.contains_key(&path) {
            path_map.insert(path.clone(), NameMap::new());
        }
        let name_map = path_map.get_mut(&path).unwrap();
        name_map.insert(
            parsed_cookie.name().to_string(),
            parsed_cookie.value().to_string(),
        );
    }
    println!("set cookies: {:#?}", map);
}

impl Store {
    pub fn create(filepath: &str) -> Self {
        let path = Path::new(filepath);
        let file = if path.exists() {
            match File::open(path) {
                Ok(file) => file,
                Err(_) => panic!("File opening failed."),
            }
        } else {
            match File::create(path) {
                Ok(file) => file,
                Err(_) => panic!("File creation failed."),
            }
        };
        let reader = BufReader::new(file);
        let map: DomainMap = match serde_json::from_reader(reader) {
            Ok(map) => map,
            Err(err) => {
                println!("File loading failed: {:#?}", err);
                DomainMap::new()
            }
        };
        Store {
            map: Mutex::new(map),
        }
    }
}

impl CookieStore for Store {
    fn set_cookies(
        &self,
        cookie_headers: &mut dyn Iterator<Item = &reqwest::header::HeaderValue>,
        url: &Url,
    ) {
        let mut map = self.map.lock().unwrap();
        set_cookies(&mut map, cookie_headers, url);
    }

    fn cookies(&self, url: &Url) -> Option<reqwest::header::HeaderValue> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::Store;
    use reqwest::{redirect::Policy, Client};

    #[tokio::test]
    async fn test_set_cookies() {
        let store = Store::create("test.json");
        let c = Client::builder()
            .redirect(Policy::limited(10))
            .cookie_store(true)
            .cookie_provider(store.into())
            .build();
        let c = match c {
            Ok(c) => c,
            Err(_) => panic!("Client creation failed."),
        };
        let res = match c.get("https://www.baidu.com").send().await {
            Ok(res) => res,
            Err(_) => panic!("Request failed."),
        };
        assert_eq!(res.status(), 200);
    }
}
