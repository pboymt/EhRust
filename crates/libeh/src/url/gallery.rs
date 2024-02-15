use crate::utils::regex::regex;

#[derive(Debug, Clone)]
pub struct GalleryBuilder {
    pub gid: isize,
    pub token: String,
    pub p: isize,
}

impl GalleryBuilder {
    pub fn new(gid: isize, token: String) -> Self {
        Self { gid, token, p: 0 }
    }

    pub fn page(&mut self, p: isize) -> &mut Self {
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
                let Ok(gid) = caps["gid"].parse::<isize>() else {
                    return Err(format!("Failed to parse gallery gid: {}", s));
                };
                gid
            },
            token: caps["token"].to_string(),
            p: 0,
        })
    }
}
