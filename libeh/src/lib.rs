use reqwest::Client;
use std::error::Error;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

// e-hentai.org 104.20.134.21
pub async fn web() -> Result<String, Box<dyn Error + Send + Sync>> {
    let client = Client::builder()
        .tls_sni(false)
        .danger_accept_invalid_certs(true)
        .build()?;
    let res = client
        .get("https://104.20.134.21")
        .header("Host", "e-hentai.org")
        .send()
        .await?;
    let text = res.text().await?;
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
