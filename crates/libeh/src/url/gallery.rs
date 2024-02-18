use crate::utils::regex::regex;

#[derive(Debug, Clone)]
pub struct GalleryBuilder {
    pub gid: i64,
    pub token: String,
    pub p: i64,
}

impl GalleryBuilder {
    pub fn new(gid: i64, token: String) -> Self {
        Self { gid, token, p: 0 }
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
