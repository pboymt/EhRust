use reqwest::Url;

use crate::{dto::site::Site, utils::regex::regex};

#[derive(Debug, Clone)]
pub struct GalleryBuilder {
    pub site: Site,
    pub gid: i64,
    pub token: String,
    pub p: i64,
}

impl GalleryBuilder {
    pub fn new(site: Site, gid: i64, token: &str) -> Self {
        Self {
            site,
            gid,
            token: token.to_string(),
            p: 0,
        }
    }

    pub fn page(&mut self, p: i64) -> &mut Self {
        self.p = p;
        self
    }
}

impl GalleryBuilder {
    pub fn parse(s: String) -> Result<Self, String> {
        let p = regex(
            r"https?://(?<site>e-hentai.org|exhentai.org)/(?:g|mpv)/(?<gid>\d+)/(?<token>[0-9a-f]{10})",
        )?;
        let Some(caps) = p.captures(&s) else {
            return Err(format!("Failed to parse gallery url: {}", s));
        };
        Ok(Self {
            site: Site::from(caps["site"].to_string()),
            gid: {
                let Ok(gid) = caps["gid"].parse::<i64>() else {
                    return Err(format!("Failed to parse gallery gid: {}", s));
                };
                gid
            },
            token: caps["token"].to_string(),
            p: 0,
        })
    }
}

impl From<GalleryBuilder> for Url {
    fn from(builder: GalleryBuilder) -> Self {
        let mut url: Url = builder.site.into();
        url.set_path(&format!("g/{}/{}/", builder.gid, builder.token));
        if builder.p > 0 {
            url.set_query(Some(&format!("p={}", builder.p)));
        }
        url
    }
}
impl From<&GalleryBuilder> for Url {
    fn from(builder: &GalleryBuilder) -> Self {
        let mut url: Url = builder.site.into();
        url.set_path(&format!("g/{}/{}/", builder.gid, builder.token));
        if builder.p > 0 {
            url.set_query(Some(&format!("p={}", builder.p)));
        }
        url
    }
}

impl ToString for GalleryBuilder {
    fn to_string(&self) -> String {
        let url = Url::from(self);
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use tokio::{fs::File, io::AsyncWriteExt};

    use crate::{
        client::{client::EhClient, config::EhClientConfig, proxy::EhClientProxy},
        dto::site::Site,
        url::gallery::GalleryBuilder,
    };

    #[test]
    fn test_gallery_builder() -> Result<(), Box<dyn std::error::Error>> {
        let url = GalleryBuilder::parse("https://e-hentai.org/g/2519745/76939e430f/".into())?;
        assert_eq!(url.gid, 2519745);
        assert_eq!(url.token, "76939e430f");
        Ok(())
    }

    #[tokio::test]
    async fn test_gallery_builder_request() -> Result<(), Box<dyn std::error::Error>> {
        let gallery_builder = GalleryBuilder::new(Site::Eh, 2519745, "76939e430f");
        let proxy = EhClientProxy::new("http", "127.0.0.1", 7890);
        let config = EhClientConfig {
            site: Site::Eh,
            proxy: Some(proxy),
            auth: None,
        };
        let client = EhClient::new(config);
        let text = client.get_html(gallery_builder.into()).await?;
        let mut cwd = std::env::current_dir().unwrap();
        cwd.push("../../samples/gallery.html");
        println!("File: {}", cwd.display());
        let file = File::create(cwd).await;
        let mut file = match file {
            Ok(file) => file,
            Err(err) => panic!("Error: {}", err),
        };
        let result = file.write(text.as_bytes()).await;
        assert!(result.is_ok());
        Ok(())
    }
}
