use serde::{Deserialize, Serialize};

use crate::dto::site::Site;

use super::{auth::EhClientAuth, proxy::EhClientProxy};

/// E-Hentai/ExHentai 客户端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EhClientConfig {
    /// 站点类型
    pub site: Site,
    /// 代理设置，默认为 None
    pub proxy: Option<EhClientProxy>,
    /// 用户身份验证设置，默认为 None
    pub auth: Option<EhClientAuth>,
}

impl Default for EhClientConfig {
    /// 创建一个默认的 EhClientConfig 实例
    fn default() -> Self {
        EhClientConfig {
            site: Site::Eh,
            proxy: None,
            auth: None,
        }
    }
}

mod tests {

    #[test]
    fn config_to_json() {
        use crate::client::{config::EhClientConfig, proxy::EhClientProxy};
        let mut config = EhClientConfig::default();
        let proxy = EhClientProxy::new("http", "127.0.0.1", 7890);
        config.proxy = Some(proxy);
        let result = serde_json::to_string(&config);
        assert!(result.is_ok());
        println!("{}", result.unwrap());
    }

    #[test]
    fn config_to_yaml() {
        use crate::client::{auth::EhClientAuth, config::EhClientConfig, proxy::EhClientProxy};
        let mut config = EhClientConfig::default();
        let proxy = EhClientProxy::new("http", "127.0.0.1", 7890);
        config.proxy = Some(proxy);
        let auth = EhClientAuth::new("", "", Some(""));
        config.auth = Some(auth);
        let result = serde_yaml::to_string(&config);
        assert!(result.is_ok());
        println!("{}", result.unwrap());
    }

    #[test]
    fn config_from_json() {
        use crate::client::config::EhClientConfig;
        let config = r#"
{
    "site": "ex",
    "proxy": {
        "protocol": "http",
        "host": "127.0.0.1",
        "port": 7890
    }
}
"#;
        let result = serde_json::from_str::<EhClientConfig>(config);
        assert!(result.is_ok());
        println!("{:?}", result.unwrap());
    }

    #[test]
    fn config_from_yaml() {
        use crate::client::config::EhClientConfig;
        let config = r#"
site: ex
proxy:
  protocol: http
  host: 127.0.0.1
  port: 7890
"#;
        let result = serde_yaml::from_str::<EhClientConfig>(config);
        assert!(result.is_ok());
        println!("{:?}", result.unwrap());
    }
}
