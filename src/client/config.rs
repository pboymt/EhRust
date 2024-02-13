use crate::url::enums::Site;

use super::{auth::EhClientAuth, proxy::EhClientProxy};

/// E-Hentai/ExHentai 客户端配置
#[derive(Debug, Clone)]
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
