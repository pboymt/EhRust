use hyper::{Client, Uri};
use hyper_tls::HttpsConnector;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub async fn web(url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let uri: Uri = url.parse()?;
    let res = client.get(uri).await?;
    Ok(res.status().to_string())
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
