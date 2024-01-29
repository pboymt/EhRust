use reqwest::{Client, Proxy, Url};
use serde::de::DeserializeOwned;

#[derive(Debug, Clone)]
pub struct EhClientProxy(String, String, i32);

impl EhClientProxy {
    pub fn new(schema: &str, host: &str, port: i32) -> Self {
        EhClientProxy(schema.to_string(), host.to_string(), port)
    }
}

#[derive(Debug, Clone)]
pub struct EhClient {
    client: Client,
}

impl EhClient {
    pub fn new(proxy: Option<EhClientProxy>) -> Self {
        let mut builder = Client::builder();
        if let Some(proxy) = proxy.clone() {
            let proxy = Proxy::all(format!("{}://{}:{}", proxy.0, proxy.1, proxy.2));
            if let Ok(proxy) = proxy {
                builder = builder.proxy(proxy);
            }
        }
        let client = builder.build();
        match client {
            Ok(client) => EhClient { client },
            Err(err) => panic!("Error: {}", err),
        }
    }

    pub async fn get_html(&self, url: Url) -> Result<String, String> {
        let res: Result<reqwest::Response, reqwest::Error> = self.client.get(url).send().await;
        let res = match res {
            Ok(res) => res,
            Err(err) => {
                return Err(format!("Error: {}", err));
            }
        };
        let text = match res.text().await {
            Ok(text) => text,
            Err(err) => {
                return Err(format!("Error: {}", err));
            }
        };
        Ok(text)
    }

    pub async fn get_json<T>(&self, url: Url) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        let res: Result<reqwest::Response, reqwest::Error> = self.client.get(url).send().await;
        let res = match res {
            Ok(res) => res,
            Err(err) => {
                return Err(format!("Error: {}", err));
            }
        };
        let json = match res.json::<T>().await {
            Ok(json) => json,
            Err(err) => {
                return Err(format!("Error: {}", err));
            }
        };
        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use tokio::{fs::File, io::AsyncWriteExt};

    use super::{EhClient, EhClientProxy};

    #[tokio::test]
    async fn test_eh_client() {
        let proxy = EhClientProxy::new("http", "127.0.0.1", 7890);
        let client = EhClient::new(Some(proxy));
        let res = client.client.get("https://e-hentai.org").send().await;
        match res {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                let text = res.text().await;
                let text = match text {
                    Ok(text) => text,
                    Err(err) => panic!("Error: {}", err),
                };
                let file = File::create("samples/index.html").await;
                let mut file = match file {
                    Ok(file) => file,
                    Err(err) => panic!("Error: {}", err),
                };
                let result = file.write(text.as_bytes()).await;
                assert!(result.is_ok());
            }
            Err(err) => panic!("Error: {}", err),
        }
    }
}
