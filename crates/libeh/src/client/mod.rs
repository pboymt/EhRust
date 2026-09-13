//! 可配置的 E-Hentai/ExHentai 客户端。
//!
//! - [`auth`](crate::client::auth)：认证信息（Cookie 三件套）与环境变量读取；
//! - [`config`](crate::client::config)：站点/代理/认证的组合配置（YAML/JSON/环境变量）；
//! - [`proxy`](crate::client::proxy)：代理设置；
//! - [`client`](crate::client::client)：[`EhClient`](crate::client::client::EhClient) —— HTTP 请求与 api.php 封装。
//!
//! 典型构造流程见 [`EhClient::try_new`](crate::client::client::EhClient::try_new)。
pub mod auth;
#[allow(clippy::module_inception)] // client::client 命名沿用上游历史结构
pub mod client;
pub mod config;
pub mod proxy;
