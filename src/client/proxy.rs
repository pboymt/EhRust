/// EhClient 代理
#[derive(Debug, Clone)]
pub struct EhClientProxy(String, String, i32);

impl EhClientProxy {
    /// 创建一个新的 EhClientProxy 实例
    pub fn new(schema: &str, host: &str, port: i32) -> Self {
        EhClientProxy(schema.to_string(), host.to_string(), port)
    }
}

impl ToString for EhClientProxy {
    /// 将 EhClientProxy 转换为 URL 字符串
    fn to_string(&self) -> String {
        format!("{}://{}:{}", self.0, self.1, self.2)
    }
}
